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

impl VcsTrait for Jj {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("log", "--color", "never", "--no-pager")
            .await
            .and_then(|status| {
                let branch = status
                    .lines()
                    .next()?
                    .split_whitespace()
                    .rev()
                    .nth(1)?
                    .trim();

                let s = &branch[..branch.len() - 1];
                Some(Chunk::new("jj ⎇", SmolStr::new(s)))
            })
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let status = jj!("status", "--color", "never", "--no-pager").await?;
        let working_copy = status
            .lines()
            .filter(|l| l.starts_with("Working copy") && l.contains("(@)"))
            .collect::<SmallVec<[_; 8]>>()
            .into_iter()
            .next()?;
        let tokens = working_copy
            .split_whitespace()
            .collect::<SmallVec<[_; 8]>>();
        if tokens.len() >= 6 {
            let change_id = tokens[4];
            let commit_id = tokens[5];
            Some(Chunk::new(
                "⭑",
                format_smolstr!("{} {}", change_id, commit_id),
            ))
        } else {
            None
        }
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("status", "--color", "never", "--no-pager")
            .await
            .and_then(|status| {
                let icons = merge_icons(
                    status
                        .lines()
                        .filter(|l| &l[1..2] == " ")
                        .map(|line| line.parse::<StatusIcon<Jj>>().unwrap())
                        .collect::<SmallVec<[_; 8]>>(),
                );
                if icons.is_empty() {
                    return None;
                }
                Some(Chunk::info(icons))
            })
    }

    async fn worktree(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        jj!("log", "--color", "never", "--no-pager")
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
