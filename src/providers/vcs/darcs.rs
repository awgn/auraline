use std::path::Path;

use crate::{chunk::Chunk, options::Options, providers::vcs::VcsTrait};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Darcs;

impl VcsTrait for Darcs {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }

    async fn worktree(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        todo!()
    }
}
