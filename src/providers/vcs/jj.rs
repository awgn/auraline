use crate::cmd::CMD;
use crate::providers::vcs::VcsTrait;
use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr};
use std::path::Path;

macro_rules! jj {
    ( $( $x:expr ),* ) => {
        CMD.exec("jj", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jj;

impl VcsTrait for Jj {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn commit(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
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

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
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
