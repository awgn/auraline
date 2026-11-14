use smol_str::SmolStr;

use crate::{
    chunk::{Adjoin, Chunk},
    options::Options,
    style::to_superscript,
};

fn effective_shlvl(shlvl_raw: Option<&str>, in_multiplexer: bool) -> Option<u32> {
    let shlvl = shlvl_raw?.trim().parse::<u32>().ok()?;
    let baseline = if in_multiplexer { 2 } else { 1 };
    (shlvl > baseline).then(|| shlvl - baseline + 1)
}

#[inline]
fn has_non_empty_env(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty())
}

pub async fn show(opts: &Options) -> Option<Chunk<Adjoin<&'static str, SmolStr>>> {
    if !opts.shlvl {
        return None;
    }

    let shlvl_raw = std::env::var("SHLVL").ok();
    let in_multiplexer =
        has_non_empty_env("TMUX") || has_non_empty_env("ZELLIJ") || has_non_empty_env("STY");

    let depth = effective_shlvl(shlvl_raw.as_deref(), in_multiplexer)?;
    let mut buf = itoa::Buffer::new();
    Some(Chunk::info(Adjoin((
        opts.select_str("↳", "󰆍"),
        to_superscript(buf.format(depth)),
    ))))
}

#[cfg(test)]
mod tests {
    use super::effective_shlvl;

    #[test]
    fn test_effective_shlvl_standard_and_multiplexer() {
        assert_eq!(effective_shlvl(None, false), None);
        assert_eq!(effective_shlvl(Some("0"), false), None);
        assert_eq!(effective_shlvl(Some("1"), false), None);
        assert_eq!(effective_shlvl(Some("2"), false), Some(2));
        assert_eq!(effective_shlvl(Some("3"), false), Some(3));

        // Inside tmux / zellij / screen, pane root shell is typically SHLVL=2 (or 1).
        assert_eq!(effective_shlvl(Some("1"), true), None);
        assert_eq!(effective_shlvl(Some("2"), true), None);
        assert_eq!(effective_shlvl(Some("3"), true), Some(2));
        assert_eq!(effective_shlvl(Some("4"), true), Some(3));
    }
}
