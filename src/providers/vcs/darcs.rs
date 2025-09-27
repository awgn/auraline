use crate::{chunk::Chunk, options::Options, providers::vcs::VcsTrait};
use crate::{
    cmd::CMD,
    providers::vcs::{merge_icons, StatusIcon},
};
use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};
use std::{path::Path, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Darcs;

macro_rules! darcs {
    ( $( $x:expr ),* ) => {
        CMD.exec("darcs", [$( $x ),*])
    };
}

impl VcsTrait for Darcs {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        darcs!("log", "--last", "1", "--summary")
            .await?
            .lines()
            .next()?
            .split_whitespace()
            .nth(1)
            .map(|s| Chunk::new("⭑", s.trim().to_smolstr()))
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        darcs!("whatsnew", "-s")
            .await
            .filter(|s| !s.is_empty())
            .map(|s| {
                Chunk::info(merge_icons(
                    s.lines()
                        .map(|l| l.parse::<StatusIcon<Darcs>>().unwrap())
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

impl FromStr for StatusIcon<Darcs> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.split_whitespace().next();
        match chars {
            Some("R") => Ok(StatusIcon::new("−")),
            Some("A") => Ok(StatusIcon::new("✚")),
            Some("M") => Ok(StatusIcon::new("●")),
            Some("F") => Ok(StatusIcon::new("→")),
            Some("T") => Ok(StatusIcon::new("→")),
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}
