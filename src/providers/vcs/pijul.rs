use crate::cmd::CMD;
use std::path::Path;
use std::str::FromStr;

use crate::providers::vcs::{merge_icons, StatusIcon};
use crate::{chunk::Chunk, options::Options};
use smol_str::{SmolStr, ToSmolStr};

macro_rules! pijul {
    ( $( $x:expr ),* ) => {
        CMD.exec("pijul", [$( $x ),*])
    };
}

struct Pijul;
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

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn commit(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    let output = pijul!("log", "--limit", "1").await?;
    let change = output.lines().next()?;
    let change_id = change.split_whitespace().nth(1)?;
    Some(Chunk::new("⭑", change_id.to_smolstr()))
}

pub async fn worktree(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn branch(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    pijul!("channel")
        .await?
        .lines()
        .find(|l| l.starts_with('*'))
        .map(|s| Chunk::new("pijul ⎇", s[1..].trim().to_smolstr()))
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    pijul!("diff", "--short")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| {
            Chunk::info(merge_icons(
                s.lines()
                    .map(|l| l.parse::<StatusIcon<Pijul>>().unwrap())
                    .collect::<Vec<_>>(),
            ))
        })
}
