pub mod git;
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
        Some(VcsKind::Git) => git::git_divergence(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn describe(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_describe(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn commit(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_commit(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn worktree(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_worktree(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn stash(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_stash(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn branch(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_branch(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
        None => None,
    }
}

pub async fn status(
    opts: &Options,
    vcs: Shared<Pin<Box<dyn Future<Output = Option<VcsKind>> + Send>>>,
) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some(VcsKind::Git) => git::git_status(opts).await,
        Some(VcsKind::Hg) => None,
        Some(VcsKind::Pijul) => None,
        Some(VcsKind::Jj) => None,
        Some(VcsKind::Darcs) => None,
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
