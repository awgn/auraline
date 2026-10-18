use crate::cmd::CMD;
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::{chunk::Chunk, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr};
use std::path::Path;
use std::str::FromStr;

macro_rules! jj {
    ( $( $x:expr ),* ) => {
        CMD.exec("jj", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jj;

fn parse_jj_branch(line: &str) -> Option<SmolStr> {
    let branch = line.split_whitespace().rev().nth(1)?.trim();
    let s = branch
        .strip_suffix('*')
        .or_else(|| branch.strip_suffix('?'))
        .unwrap_or(branch);
    (!s.is_empty()).then(|| SmolStr::new(s))
}

impl VcsTrait for Jj {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("log", "--limit", "1", "--color", "never", "--no-pager")
            .await
            .and_then(|status| {
                let first_line = status.lines().next()?;
                let s = parse_jj_branch(first_line)?;
                Some(Chunk::new("jj ⎇", s))
            })
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let status = jj!("status", "--color", "never", "--no-pager").await?;
        let working_copy = status
            .lines()
            .find(|l| l.starts_with("Working copy") && l.contains("(@)"))?;
        let mut tokens = working_copy.split_whitespace();
        let change_id = tokens.nth(4)?;
        let commit_id = tokens.next()?;
        Some(Chunk::new(
            "⭑",
            format_smolstr!("{} {}", change_id, commit_id),
        ))
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("status", "--color", "never", "--no-pager")
            .await
            .and_then(|status| {
                let icons = merge_icons(
                    status
                        .lines()
                        .filter(|l| l.as_bytes().get(1) == Some(&b' '))
                        .filter_map(|line| line.parse::<StatusIcon<Jj>>().ok())
                        .collect::<SmallVec<[_; 8]>>(),
                );
                if icons.is_empty() {
                    return None;
                }
                Some(Chunk::info(icons))
            })
    }

    async fn worktree(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("log", "--limit", "1", "--color", "never", "--no-pager")
            .await
            .and_then(|status| {
                let mut tokens = status.lines().next()?.split_whitespace();

                if tokens.clone().count() == 8 {
                    tokens.nth(5).map(|t| Chunk::new("¶", SmolStr::new(t)))
                } else {
                    None
                }
            })
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }
}

impl FromStr for StatusIcon<Jj> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.split_whitespace().next();
        match chars {
            Some("A") => Ok(StatusIcon::new("✚")),
            Some("R") => Ok(StatusIcon::new("→")),
            Some("M") => Ok(StatusIcon::new("●")),
            Some("C") => Ok(StatusIcon::new("⊂")),
            Some("D") => Ok(StatusIcon::new("−")),
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_jj_branch;

    #[test]
    fn test_parse_jj_branch_safe() {
        assert_eq!(
            parse_jj_branch("@  qpvuntsm user@example.com 2026-10-18 07:00:00 main* 12345678")
                .as_deref(),
            Some("main")
        );
        assert_eq!(
            parse_jj_branch("@  qpvuntsm user@example.com 2026-10-18 07:00:00 main 12345678")
                .as_deref(),
            Some("main")
        );
        // Multi-byte UTF-8 character at the end must not panic
        assert_eq!(
            parse_jj_branch("@  qpvuntsm user@example.com 2026-10-18 07:00:00 🚀 12345678")
                .as_deref(),
            Some("🚀")
        );
        // Single character token must not return empty string
        assert_eq!(parse_jj_branch("@ * 12345678"), None);
    }
}
