pub mod darcs;
pub mod git;
pub mod hg;
pub mod jj;
pub mod pijul;

use std::{future::Future, path::PathBuf, pin::Pin};

use crate::{chunk::Chunk, options::Options, style::to_superscript};
use futures::future::Shared;
use itertools::Itertools;
use smol_str::{SmolStr, SmolStrBuilder};
use tokio::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsKind {
    Git,
    Hg,
    Pijul,
    Jj,
    Darcs,
}

type FutureVsc = Pin<Box<dyn Future<Output = Option<(VcsKind, PathBuf)>> + Send>>;

pub async fn divergence(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some((VcsKind::Git, ref path)) => git::divergence(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::divergence(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::divergence(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::divergence(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::divergence(opts, path).await,
        None => None,
    }
}

pub async fn commit(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some((VcsKind::Git, ref path)) => git::commit(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::commit(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::commit(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::commit(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::commit(opts, path).await,
        None => None,
    }
}

pub async fn worktree(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    let vcs = vcs.await;
    match vcs {
        Some((VcsKind::Git, ref path)) => git::worktree(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::worktree(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::worktree(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::worktree(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::worktree(opts, path).await,
        None => None,
    }
}

pub async fn stash(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some((VcsKind::Git, ref path)) => git::stash(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::stash(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::stash(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::stash(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::stash(opts, path).await,
        None => None,
    }
}

pub async fn branch(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some((VcsKind::Git, ref path)) => git::branch(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::branch(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::branch(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::branch(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::branch(opts, path).await,
        None => None,
    }
}

pub async fn status(opts: &Options, vcs: Shared<FutureVsc>) -> Option<Chunk<SmolStr>> {
    match vcs.await {
        Some((VcsKind::Git, ref path)) => git::status(opts, path).await,
        Some((VcsKind::Hg, ref path)) => hg::status(opts, path).await,
        Some((VcsKind::Pijul, ref path)) => pijul::status(opts, path).await,
        Some((VcsKind::Jj, ref path)) => jj::status(opts, path).await,
        Some((VcsKind::Darcs, ref path)) => darcs::status(opts, path).await,
        None => None,
    }
}

pub async fn detect_vcs(start: PathBuf) -> Option<(VcsKind, PathBuf)> {
    let mut dir = start.canonicalize().ok()?;
    loop {
        if fs::metadata(dir.join(".jj")).await.is_ok() {
            return Some((VcsKind::Jj, dir));
        }
        if fs::metadata(dir.join(".git")).await.is_ok() {
            return Some((VcsKind::Git, dir));
        }
        if fs::metadata(dir.join(".hg")).await.is_ok() {
            return Some((VcsKind::Hg, dir));
        }
        if fs::metadata(dir.join(".pijul")).await.is_ok() {
            return Some((VcsKind::Pijul, dir));
        }
        if fs::metadata(dir.join("_darcs")).await.is_ok() {
            return Some((VcsKind::Darcs, dir));
        }

        if !dir.pop() {
            break;
        }
    }

    None
}
pub fn merge_icons<T: AsRef<str>>(icons: Vec<T>) -> SmolStr {
    let mut builder = SmolStrBuilder::new();
    icons
        .iter()
        .map(AsRef::<str>::as_ref)
        .filter(|i| !i.is_empty()) // directly filter out empty icons
        .sorted()
        .chunk_by(|icon| *icon) // group by str value using chunk_by
        .into_iter()
        .for_each(|(icon, group)| builder.push_str(&render_icon((icon, group.count()))));

    builder.finish()
}

pub fn render_icon<T: AsRef<str>>((icon, n): (T, usize)) -> SmolStr {
    let mut builder = SmolStrBuilder::new();
    if n == 1 {
        builder.push_str(icon.as_ref());
    } else {
        let mut buffer = itoa::Buffer::new();
        let numb = buffer.format(n);
        builder.push_str(icon.as_ref());
        builder.push_str(&to_superscript(numb));
    }
    builder.finish()
}
