use crate::cmd::CMD;
use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr};
use std::path::Path;

macro_rules! jj {
    ( $( $x:expr ),* ) => {
        CMD.exec("jj", [$( $x ),*])
    };
}

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    // not supported
    None
}

pub async fn commit(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    let status = jj!("status", "--color", "never").await?;
    let working_copy = status
        .lines()
        .filter(|l| l.starts_with("Working copy") && l.contains("(@)"))
        .collect::<Vec<_>>()
        .into_iter()
        .next()?;
    let tokens = working_copy.split_whitespace().collect::<Vec<_>>();
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

pub async fn worktree(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    // not supported
    None
}

pub async fn branch(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}
