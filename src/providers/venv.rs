use smol_str::SmolStr;
use std::path::Path;

use crate::{chunk::Chunk, options::Options};

fn clean_prompt_label(raw: &str) -> Option<SmolStr> {
    let mut s = raw.trim();
    s = s.trim_matches(['"', '\'']).trim();
    if let Some(inner) = s.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        s = inner.trim();
    }
    (!s.is_empty()).then(|| SmolStr::new(s))
}

fn parse_pyvenv_prompt(cfg_content: &str) -> Option<SmolStr> {
    for line in cfg_content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("prompt") {
            let rest = rest.trim_start();
            if let Some(val) = rest.strip_prefix('=') {
                return clean_prompt_label(val);
            }
        }
    }
    None
}

fn resolve_venv_from(
    virtual_env: Option<&str>,
    virtual_env_prompt: Option<&str>,
    conda_env: Option<&str>,
    pixi_env: Option<&str>,
    pixi_project: Option<&str>,
) -> Option<SmolStr> {
    if let Some(venv_dir) = virtual_env.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(p) = virtual_env_prompt.and_then(clean_prompt_label) {
            return Some(p);
        }
        let path = Path::new(venv_dir);
        if let Ok(cfg) = std::fs::read_to_string(path.join("pyvenv.cfg")) {
            if let Some(p) = parse_pyvenv_prompt(&cfg) {
                return Some(p);
            }
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if !name.is_empty() {
                return Some(SmolStr::new(name));
            }
        }
    }

    if let Some(conda) = conda_env.and_then(clean_prompt_label) {
        return Some(conda);
    }

    if let Some(pixi) = pixi_env.and_then(clean_prompt_label) {
        if pixi != "default" {
            return Some(pixi);
        }
        if let Some(proj) = pixi_project.and_then(clean_prompt_label) {
            return Some(proj);
        }
        return Some(pixi);
    }

    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.venv {
        return None;
    }

    let virtual_env = std::env::var("VIRTUAL_ENV").ok();
    let virtual_env_prompt = std::env::var("VIRTUAL_ENV_PROMPT").ok();
    let conda_env = std::env::var("CONDA_DEFAULT_ENV").ok();
    let pixi_env = std::env::var("PIXI_ENVIRONMENT_NAME").ok();
    let pixi_project = std::env::var("PIXI_PROJECT_NAME").ok();

    let name = resolve_venv_from(
        virtual_env.as_deref(),
        virtual_env_prompt.as_deref(),
        conda_env.as_deref(),
        pixi_env.as_deref(),
        pixi_project.as_deref(),
    )?;

    Some(Chunk::new(opts.select_str("⏣", ""), name))
}

#[cfg(test)]
mod tests {
    use super::{clean_prompt_label, parse_pyvenv_prompt, resolve_venv_from};

    #[test]
    fn test_clean_prompt_and_pyvenv_cfg() {
        assert_eq!(clean_prompt_label("(my-env) ").as_deref(), Some("my-env"));
        assert_eq!(
            parse_pyvenv_prompt("home = /usr/bin\nprompt = 'data-sci'\n").as_deref(),
            Some("data-sci")
        );
    }

    #[test]
    fn test_resolve_venv_sources_and_precedence() {
        let dir = std::env::temp_dir().join(format!("auraline-test-venv-{}", std::process::id()));
        let venv_path = dir.join(".venv");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&venv_path).unwrap();

        // Fallback to directory basename when no prompt override exists.
        assert_eq!(
            resolve_venv_from(venv_path.to_str(), None, None, None, None).as_deref(),
            Some(".venv")
        );

        // Reads `prompt = ...` from `pyvenv.cfg`.
        std::fs::write(venv_path.join("pyvenv.cfg"), "prompt = \"uv-app\"\n").unwrap();
        assert_eq!(
            resolve_venv_from(venv_path.to_str(), None, None, None, None).as_deref(),
            Some("uv-app")
        );

        // `VIRTUAL_ENV_PROMPT` overrides `pyvenv.cfg`.
        assert_eq!(
            resolve_venv_from(venv_path.to_str(), Some("(custom) "), None, None, None).as_deref(),
            Some("custom")
        );

        // Conda & Pixi fallbacks.
        assert_eq!(
            resolve_venv_from(None, None, Some("pytorch"), None, None).as_deref(),
            Some("pytorch")
        );
        assert_eq!(
            resolve_venv_from(None, None, None, Some("default"), Some("robotics")).as_deref(),
            Some("robotics")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
