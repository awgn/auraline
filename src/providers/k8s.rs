use smol_str::{format_smolstr, SmolStr};
use std::path::{Path, PathBuf};

use crate::{chunk::Chunk, options::Options};

fn parse_yaml_scalar(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"')?;
        let val = rest[..end].trim();
        return (!val.is_empty()).then_some(val);
    }
    if let Some(rest) = s.strip_prefix('\'') {
        let end = rest.find('\'')?;
        let val = rest[..end].trim();
        return (!val.is_empty()).then_some(val);
    }
    // Unquoted scalar: strip inline ` #...` comments.
    let val = match s.find(" #") {
        Some(idx) => s[..idx].trim_end(),
        None => s,
    };
    (!val.is_empty() && val != "~" && val != "null").then_some(val)
}

fn parse_current_context(content: &str) -> Option<&str> {
    for line in content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("current-context:") {
            return parse_yaml_scalar(rest);
        }
    }
    None
}

/// Finds `(found_context, optional_namespace)` for `target_ctx` inside a kubeconfig YAML document.
fn find_context_namespace<'a>(content: &'a str, target_ctx: &str) -> (bool, Option<&'a str>) {
    let mut in_contexts = false;
    let mut list_indent: Option<usize> = None;
    let mut item_name: Option<&'a str> = None;
    let mut item_ns: Option<&'a str> = None;
    let mut in_context_block = false;
    let mut context_field_indent: Option<usize> = None;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = raw_line.len() - raw_line.trim_start_matches(' ').len();

        if !in_contexts {
            if indent == 0 && trimmed == "contexts:" {
                in_contexts = true;
            }
            continue;
        }

        // A 0-indented non-list line terminates the top-level `contexts:` block.
        if indent == 0 && !trimmed.starts_with('-') {
            if item_name == Some(target_ctx) {
                return (true, item_ns);
            }
            break;
        }

        if let Some(after_dash) = trimmed.strip_prefix('-') {
            if (after_dash.is_empty() || after_dash.starts_with(' '))
                && list_indent.is_none_or(|li| indent == li)
            {
                if item_name == Some(target_ctx) {
                    return (true, item_ns);
                }
                list_indent = Some(indent);
                item_name = None;
                item_ns = None;
                in_context_block = false;
                context_field_indent = None;

                let rest = after_dash.trim_start();
                if rest == "context:" {
                    in_context_block = true;
                } else if let Some(val) = rest.strip_prefix("name:") {
                    item_name = parse_yaml_scalar(val);
                }
                continue;
            }
        }

        let Some(li) = list_indent else {
            continue;
        };
        if indent <= li {
            continue;
        }

        let item_key_indent = li + 2;
        if indent == item_key_indent {
            in_context_block = false;
            context_field_indent = None;
            if let Some(val) = trimmed.strip_prefix("name:") {
                item_name = parse_yaml_scalar(val);
            } else if trimmed == "context:" {
                in_context_block = true;
            }
        } else if in_context_block && indent == *context_field_indent.get_or_insert(indent) {
            if let Some(val) = trimmed.strip_prefix("namespace:") {
                item_ns = parse_yaml_scalar(val);
            }
        }
    }

    if item_name == Some(target_ctx) {
        return (true, item_ns);
    }

    (false, None)
}

fn kubeconfig_paths() -> Vec<PathBuf> {
    if let Some(env_val) = std::env::var_os("KUBECONFIG") {
        let paths: Vec<PathBuf> = std::env::split_paths(&env_val)
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
        if !paths.is_empty() {
            return paths;
        }
    }
    std::env::home_dir()
        .map(|h| vec![h.join(".kube").join("config")])
        .unwrap_or_default()
}

fn resolve_k8s_from_files(paths: &[PathBuf]) -> Option<SmolStr> {
    let mut contents = Vec::with_capacity(paths.len());
    for p in paths {
        if let Ok(text) = std::fs::read_to_string(Path::new(p)) {
            contents.push(text);
        }
    }

    let current_ctx = contents.iter().find_map(|c| parse_current_context(c))?;

    let mut namespace = None;
    for c in &contents {
        let (found, ns) = find_context_namespace(c, current_ctx);
        if found {
            namespace = ns;
            break;
        }
    }

    match namespace {
        Some(ns) if !ns.is_empty() && ns != "default" => {
            Some(format_smolstr!("{current_ctx}/{ns}"))
        }
        _ => Some(SmolStr::new(current_ctx)),
    }
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.k8s {
        return None;
    }

    let paths = kubeconfig_paths();
    let label = resolve_k8s_from_files(&paths)?;
    Some(Chunk::new(opts.select_str("⎈", "󱃾"), label))
}

#[cfg(test)]
mod tests {
    use super::{find_context_namespace, parse_current_context, resolve_k8s_from_files};

    #[test]
    fn test_parse_kubectl_standard_and_reverse_order() {
        let cfg = "\
apiVersion: v1
clusters: []
contexts:
- context:
    cluster: prod-cluster
    namespace: payments
    user: admin
  name: prod-eu
- name: kind-local
  context:
    cluster: kind
    namespace: default
current-context: prod-eu
kind: Config
";
        assert_eq!(parse_current_context(cfg), Some("prod-eu"));
        assert_eq!(
            find_context_namespace(cfg, "prod-eu"),
            (true, Some("payments"))
        );
        assert_eq!(
            find_context_namespace(cfg, "kind-local"),
            (true, Some("default"))
        );
    }

    #[test]
    fn test_multi_file_kubeconfig_merge_and_default_ns_omission() {
        let dir = std::env::temp_dir().join(format!("auraline-test-k8s-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let f1 = dir.join("base.yaml");
        let f2 = dir.join("extra.yaml");

        std::fs::write(
            &f1,
            "current-context: \"staging-cluster\" # active\ncontexts:\n  - name: local\n    context:\n      namespace: dev\n",
        )
        .unwrap();
        std::fs::write(
            &f2,
            "contexts:\n  - context:\n      cluster: stg\n      namespace: 'core-api'\n    name: \"staging-cluster\"\n",
        )
        .unwrap();

        assert_eq!(
            resolve_k8s_from_files(&[f1.clone(), f2.clone()]).as_deref(),
            Some("staging-cluster/core-api")
        );

        std::fs::write(
            &f2,
            "contexts:\n  - name: staging-cluster\n    context:\n      namespace: default\n",
        )
        .unwrap();
        assert_eq!(
            resolve_k8s_from_files(&[f1, f2]).as_deref(),
            Some("staging-cluster")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
