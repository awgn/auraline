pub mod darcs;
pub mod git;
pub mod hg;
pub mod jj;
pub mod pijul;

use std::path::{Path, PathBuf};

use crate::{chunk::Chunk, options::Options, style::to_superscript};

use crate::providers::vcs::darcs::Darcs;
use crate::providers::vcs::git::Git;
use crate::providers::vcs::hg::Hg;
use crate::providers::vcs::jj::Jj;
use crate::providers::vcs::pijul::Pijul;

use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use smallvec::SmallVec;
use smol_str::{SmolStr, SmolStrBuilder};
use tokio::fs;

#[enum_dispatch]
pub trait VcsTrait {
    async fn branch(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
    async fn commit(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
    async fn status(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
    async fn worktree(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
    async fn stash(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
    async fn divergence(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>>;
}

#[enum_dispatch(VcsTrait)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vcs {
    Git,
    Hg,
    Jj,
    Pijul,
    Darcs,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct StatusIcon<T> {
    pub value: &'static str,
    pub marker: std::marker::PhantomData<T>,
}

impl<T> StatusIcon<T> {
    pub const fn new(value: &'static str) -> Self {
        Self {
            value,
            marker: std::marker::PhantomData,
        }
    }
}

impl<T> AsRef<str> for StatusIcon<T> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

pub async fn infer_vcs(start: PathBuf) -> Option<(Vcs, PathBuf)> {
    let mut dir = start.canonicalize().ok()?;
    loop {
        if fs::metadata(dir.join(".jj")).await.is_ok() {
            return Some((Vcs::Jj(Jj), dir));
        }
        if fs::metadata(dir.join(".git")).await.is_ok() {
            return Some((Vcs::Git(Git), dir));
        }
        if fs::metadata(dir.join(".hg")).await.is_ok() {
            return Some((Vcs::Hg(Hg), dir));
        }
        if fs::metadata(dir.join(".pijul")).await.is_ok() {
            return Some((Vcs::Pijul(Pijul), dir));
        }
        if fs::metadata(dir.join("_darcs")).await.is_ok() {
            return Some((Vcs::Darcs(Darcs), dir));
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

pub fn merge_icons<T: AsRef<str>, const N: usize>(icons: SmallVec<[T; N]>) -> SmolStr {
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
