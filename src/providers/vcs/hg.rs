use smol_str::SmolStr;
use crate::{chunk::Chunk, options::Options};

pub async fn divergence(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn describe(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn commit(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn worktree(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn stash(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn branch(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn status(_: &Options) -> Option<Chunk<SmolStr>> {
    None
}
