use std::path::Path;

use crate::{chunk::Chunk, options::Options};
use smol_str::SmolStr;

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn commit(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn worktree(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn branch(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}
