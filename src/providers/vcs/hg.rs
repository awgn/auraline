use crate::cmd::CMD;
use crate::providers::vcs::merge_icons;
use crate::{chunk::Chunk, options::Options};
use smol_str::{SmolStr, ToSmolStr};
use std::path::Path;
use std::str::FromStr;
use tokio::fs;

macro_rules! hg {
    ( $( $x:expr ),* ) => {
        CMD.exec("hg", [$( $x ),*])
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct StatusIcon(pub &'static str);

impl AsRef<str> for StatusIcon {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl FromStr for StatusIcon {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next() {
            Some('A') => Ok(StatusIcon("✚")), // added
            Some('M') => Ok(StatusIcon("●")), // modified
            Some('R') => Ok(StatusIcon("✖")), // removed
            Some('!') => Ok(StatusIcon("!")), // missing
            Some('?') => Ok(StatusIcon("?")), // not tracked
            Some('C') => Ok(StatusIcon("")),  // clean
            Some('I') => Ok(StatusIcon("")),  // Ignored
            _ => Ok(StatusIcon("")),          // Unknown state
        }
    }
}

pub async fn branch(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    hg!("branch")
        .await
        .map(|s| Chunk::new("hg ⎇", s.trim().to_smolstr()))
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    hg!("status").await.filter(|s| !s.is_empty()).map(|status| {
        Chunk::info(merge_icons(
            status
                .lines()
                .map(|line| line.parse::<StatusIcon>().unwrap())
                .collect::<Vec<_>>(),
        ))
    })
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn worktree(_: &Options, base: &Path) -> Option<Chunk<SmolStr>> {
    let path = base.join(".hg").join("sharedpath");
    let sharedpath = fs::read_to_string(&path).await.ok()?;
    sharedpath
        .rfind('/')
        .map(|pos| Chunk::new("⌂", sharedpath[..pos].into()))
}

pub async fn commit(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    hg!("id").await.map(|s| Chunk::info(s.trim().to_smolstr()))
}

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}
