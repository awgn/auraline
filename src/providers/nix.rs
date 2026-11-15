use smol_str::SmolStr;

use crate::{chunk::Chunk, options::Options};

#[derive(Debug, PartialEq, Eq)]
enum EnvKind {
    Nix(SmolStr),
    Direnv,
}

fn detect_nix_or_direnv(
    in_nix_shell: Option<&str>,
    nix_name: Option<&str>,
    nix_build_top: Option<&str>,
    direnv_dir: Option<&str>,
) -> Option<EnvKind> {
    let in_nix = in_nix_shell.map(str::trim).filter(|s| !s.is_empty());
    let has_build_top = nix_build_top.is_some_and(|s| !s.trim().is_empty());

    if in_nix.is_some() || has_build_top {
        let custom_name = nix_name
            .map(str::trim)
            .filter(|n| !n.is_empty() && *n != "shell" && *n != "nix-shell");

        let label = if let Some(n) = custom_name {
            SmolStr::new(n)
        } else if let Some(mode) = in_nix {
            SmolStr::new(mode)
        } else {
            SmolStr::new_static("nix")
        };
        return Some(EnvKind::Nix(label));
    }

    if direnv_dir.is_some_and(|d| !d.trim().is_empty()) {
        return Some(EnvKind::Direnv);
    }

    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.nix {
        return None;
    }

    let in_nix_shell = std::env::var("IN_NIX_SHELL").ok();
    let nix_name = std::env::var("name").ok();
    let nix_build_top = std::env::var("NIX_BUILD_TOP").ok();
    let direnv_dir = std::env::var("DIRENV_DIR").ok();

    match detect_nix_or_direnv(
        in_nix_shell.as_deref(),
        nix_name.as_deref(),
        nix_build_top.as_deref(),
        direnv_dir.as_deref(),
    )? {
        EnvKind::Nix(label) => Some(Chunk::new(opts.select_str("✻", ""), label)),
        EnvKind::Direnv => Some(Chunk::new(
            opts.select_str("▼", "󰁿"),
            SmolStr::new_static("direnv"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_nix_or_direnv, EnvKind};
    use smol_str::SmolStr;

    #[test]
    fn test_detect_nix_shell_and_direnv() {
        assert_eq!(detect_nix_or_direnv(None, None, None, None), None);

        assert_eq!(
            detect_nix_or_direnv(Some("impure"), Some("nix-shell"), None, None),
            Some(EnvKind::Nix(SmolStr::new_static("impure")))
        );

        assert_eq!(
            detect_nix_or_direnv(Some("impure"), Some("devshell"), None, Some("-/tmp/proj")),
            Some(EnvKind::Nix(SmolStr::new_static("devshell")))
        );

        assert_eq!(
            detect_nix_or_direnv(None, None, Some("/tmp/nix-shell.123"), None),
            Some(EnvKind::Nix(SmolStr::new_static("nix")))
        );

        assert_eq!(
            detect_nix_or_direnv(None, None, None, Some("-/Users/nicola/proj")),
            Some(EnvKind::Direnv)
        );
    }
}
