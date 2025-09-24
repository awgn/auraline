pub mod git;
pub mod hg;
pub mod jj;
pub mod pijul;
pub mod darcs;

use std::{future::Future, path::PathBuf, pin::Pin};

use crate::{chunk::Chunk, options::Options};
use futures::future::Shared;
use smol_str::SmolStr;
use tokio::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsKind {
    Git,
    Hg,
    Pijul,
    Jj,
    Darcs,
}

pub async fn divergence(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::divergence(opts).await,
        Some(VcsKind::Hg) => hg::divergence(opts).await,
        Some(VcsKind::Pijul) => pijul::divergence(opts).await,
        Some(VcsKind::Jj) => jj::divergence(opts).await,
        Some(VcsKind::Darcs) => darcs::divergence(opts).await,
        None => None,
    }
}

pub async fn describe(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::describe(opts).await,
        Some(VcsKind::Hg) => hg::describe(opts).await,
        Some(VcsKind::Pijul) => pijul::describe(opts).await,
        Some(VcsKind::Jj) => jj::describe(opts).await,
        Some(VcsKind::Darcs) => darcs::describe(opts).await,
        None => None,
    }
}

pub async fn commit(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::commit(opts).await,
        Some(VcsKind::Hg) => hg::commit(opts).await,
        Some(VcsKind::Pijul) => pijul::commit(opts).await,
        Some(VcsKind::Jj) => jj::commit(opts).await,
        Some(VcsKind::Darcs) => darcs::commit(opts).await,
        None => None,
    }
}

pub async fn worktree(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::worktree(opts).await,
        Some(VcsKind::Hg) => hg::worktree(opts).await,
        Some(VcsKind::Pijul) => pijul::worktree(opts).await,
        Some(VcsKind::Jj) => jj::worktree(opts).await,
        Some(VcsKind::Darcs) => darcs::worktree(opts).await,
        None => None,
    }
}

pub async fn stash(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::stash(opts).await,
        Some(VcsKind::Hg) => hg::stash(opts).await,
        Some(VcsKind::Pijul) => pijul::stash(opts).await,
        Some(VcsKind::Jj) => jj::stash(opts).await,
        Some(VcsKind::Darcs) => darcs::stash(opts).await,
        None => None,
    }
}

pub async fn branch(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::branch(opts).await,
        Some(VcsKind::Hg) => hg::branch(opts).await,
        Some(VcsKind::Pijul) => pijul::branch(opts).await,
        Some(VcsKind::Jj) => jj::branch(opts).await,
        Some(VcsKind::Darcs) => darcs::branch(opts).await,
        None => None,
    }
}

pub async fn status(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::status(opts).await,
        Some(VcsKind::Hg) => hg::status(opts).await,
        Some(VcsKind::Pijul) => pijul::status(opts).await,
        Some(VcsKind::Jj) => jj::status(opts).await,
        Some(VcsKind::Darcs) => darcs::status(opts).await,
        None => None,
    }
}

pub async fn detect_vcs(start: PathBuf) -> Option<VcsKind> {
    let mut dir = start.canonicalize().ok()?;
    loop {
        if fs::metadata(dir.join(".jj")).await.is_ok() {
            return Some(VcsKind::Jj);
        }
        if fs::metadata(dir.join(".git")).await.is_ok() {
            return Some(VcsKind::Git);
        }
        if fs::metadata(dir.join(".hg")).await.is_ok() {
            return Some(VcsKind::Hg);
        }
        if fs::metadata(dir.join(".pijul")).await.is_ok() {
            return Some(VcsKind::Pijul);
        }
        if fs::metadata(dir.join("_darcs")).await.is_ok() {
            return Some(VcsKind::Darcs);
        }

        if !dir.pop() {
            break;
        }
    }

    None
}
