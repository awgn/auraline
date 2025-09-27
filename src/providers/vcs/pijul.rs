use crate::cmd::CMD;
use std::path::Path;
use std::str::FromStr;

use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::{chunk::Chunk, options::Options};
use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};

macro_rules! pijul {
    ( $( $x:expr ),* ) => {
        CMD.exec("pijul", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pijul;

impl VcsTrait for Pijul {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        pijul!("channel")
            .await?
            .lines()
            .find(|l| l.starts_with('*'))
            .map(|s| Chunk::new("pijul ⎇", s[1..].trim().to_smolstr()))
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let output = pijul!("log", "--limit", "1").await?;
        let change = output.lines().next()?;
        let change_id = change.split_whitespace().nth(1)?;
        Some(Chunk::new("⭑", change_id.to_smolstr()))
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        pijul!("diff", "--short")
            .await
            .filter(|s| !s.is_empty())
            .map(|s| {
                Chunk::info(merge_icons(
                    s.lines()
                        .map(|l| l.parse::<StatusIcon<Pijul>>().unwrap())
                        .collect::<SmallVec<[_; 8]>>(),
                ))
            })
    }

    async fn worktree(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }
}

impl FromStr for StatusIcon<Pijul> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.split_whitespace().next();
        match chars {
            Some("MV") => Ok(StatusIcon::new("→")),
            Some("D") => Ok(StatusIcon::new("−")),
            Some("UD") => Ok(StatusIcon::new("⊖")),
            Some("A") => Ok(StatusIcon::new("✚")),
            Some("SC") => Ok(StatusIcon::new("⚠")),
            Some("UC") => Ok(StatusIcon::new("!")),
            Some("M") => Ok(StatusIcon::new("●")),
            Some("R") => Ok(StatusIcon::new("◉")),
            Some("RZ") => Ok(StatusIcon::new("↺")),
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}
