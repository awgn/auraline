use crate::chunk::Chunk;
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::style::to_superscript;
use crate::{cmd::CMD, commands::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, StrExt, ToSmolStr};
use std::convert::Infallible;
use std::env;
use std::path::Path;
use std::str::FromStr;
use tokio::join;

macro_rules! git {
    ( $( $x:expr ),* ) => {
        CMD.exec("git", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Git;

impl VcsTrait for Git {
    async fn branch(&self, opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let icon = git_branch_icon(opts).await;
        let info = git_branch_name(opts).await;
        match (icon, info) {
            (None, None) => None,
            (Some(icon), None) => Some(Chunk::icon(icon)),
            (None, Some(info)) => Some(Chunk::info(info)),
            (Some(icon), Some(info)) => Some(Chunk::new(icon, info)),
        }
    }

    async fn commit(&self, opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let (name_rev, branch_name, descr) = join!(
            git_name_rev(opts),
            git_branch_name(opts),
            git_describe_cmd(opts)
        );

        match (branch_name, descr, name_rev) {
            (_, None, None) => None,
            (None, None, Some(nr)) => Some(Chunk::info(nr)),
            (Some(b), None, Some(nr)) if git_bidirectional_inclusion(&b, &nr) => None,
            (Some(_), None, Some(nr)) => Some(Chunk::info(nr)),
            (_, Some(c), _) => Some(Chunk::info(c)),
        }
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        git!("status", "--porcelain")
            .await
            .filter(|s| !s.is_empty())
            .map(|s| {
                Chunk::info(merge_icons(
                    s.lines()
                        .map(|l| l.parse::<StatusIcon<Git>>().unwrap())
                        .collect::<SmallVec<[_; 8]>>(),
                ))
            })
    }

    async fn worktree(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let path = env::current_dir().ok()?;
        let output = git!("worktree", "list").await?;
        output.lines().skip(1).find_map(|line| {
            let mut parts = line.split_whitespace();
            let worktree_path = parts.next()?;
            if path.starts_with(worktree_path) {
                parts.next()?; // skip the branch
                let name = parts.collect::<SmallVec<[_; 8]>>().join(" ");
                Some(Chunk::new("⌂", name.into()))
            } else {
                None
            }
        })
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        git!("stash", "list")
            .await
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut buffer = itoa::Buffer::new();
                let n = buffer.format(s.lines().count());
                Chunk::info(format_smolstr!("≡{}", to_superscript(n)))
            })
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let (ahead, behind) = join!(
            git!("rev-list", "--count", "HEAD@{upstream}..HEAD"),
            git!("rev-list", "--count", "HEAD..HEAD@{upstream}")
        );

        let ahead = ahead?;
        let behind = behind?;

        match (ahead.as_str(), behind.as_str()) {
            ("0" | "", "0" | "") => None,
            ("0" | "", behind) => Some(Chunk::info(format_smolstr!("↓{}", behind))),
            (ahead, "0" | "") => Some(Chunk::info(format_smolstr!("↑{}", ahead))),
            (ahead, behind) => Some(Chunk::info(format_smolstr!("↑{}↓{}", ahead, behind))),
        }
    }
}

impl FromStr for StatusIcon<Git> {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            // Unmerged states (conflicts)
            (Some('D'), Some('D')) => Ok(StatusIcon::new("✖")), // Both deleted
            (Some('A'), Some('A')) => Ok(StatusIcon::new("⧉")), // Both added
            (Some('U'), Some('U')) => Ok(StatusIcon::new("⚠")), // Both modified - warning
            (Some('A'), Some('U')) => Ok(StatusIcon::new("⊕")), // Added by us
            (Some('U'), Some('A')) => Ok(StatusIcon::new("⊞")), // Added by them
            (Some('D'), Some('U')) => Ok(StatusIcon::new("⊖")), // Deleted by us
            (Some('U'), Some('D')) => Ok(StatusIcon::new("⊟")), // Deleted by them

            // Index changes
            (Some('M'), Some(' ')) => Ok(StatusIcon::new("●")), // Modified in index only
            (Some('M'), Some('M')) => Ok(StatusIcon::new("◉")), // Modified in both
            (Some('M'), Some('D')) => Ok(StatusIcon::new("◐")), // Modified in index, deleted in worktree
            (Some('M'), Some('T')) => Ok(StatusIcon::new("◑")), // Modified in index, type changed in worktree

            (Some('A'), Some(' ')) => Ok(StatusIcon::new("✚")), // Added to index only
            (Some('A'), Some('M')) => Ok(StatusIcon::new("✛")), // Added and modified
            (Some('A'), Some('D')) => Ok(StatusIcon::new("⊕")), // Added then deleted in worktree
            (Some('A'), Some('T')) => Ok(StatusIcon::new("⊛")), // Added, type changed in worktree

            (Some('D'), Some(' ')) => Ok(StatusIcon::new("−")), // Deleted from index
            (Some('D'), Some('M')) => Ok(StatusIcon::new("∓")), // Deleted in index but modified in worktree (weird state)

            (Some('R'), Some(' ')) => Ok(StatusIcon::new("→")), // Renamed in index
            (Some('R'), Some('M')) => Ok(StatusIcon::new("⇢")), // Renamed and modified
            (Some('R'), Some('D')) => Ok(StatusIcon::new("⇥")), // Renamed then deleted
            (Some('R'), Some('T')) => Ok(StatusIcon::new("⤳")), // Renamed and type changed

            (Some('C'), Some(' ')) => Ok(StatusIcon::new("⊂")), // Copied in index
            (Some('C'), Some('M')) => Ok(StatusIcon::new("⊃")), // Copied and modified
            (Some('C'), Some('D')) => Ok(StatusIcon::new("⊄")), // Copied then deleted
            (Some('C'), Some('T')) => Ok(StatusIcon::new("⊅")), // Copied and type changed

            (Some('T'), Some(' ')) => Ok(StatusIcon::new("◈")), // Type changed in index
            (Some('T'), Some('M')) => Ok(StatusIcon::new("◊")), // Type changed and modified
            (Some('T'), Some('D')) => Ok(StatusIcon::new("⬧")), // Type changed then deleted
            (Some('T'), Some('T')) => Ok(StatusIcon::new("⬢")), // Type changed in both

            (Some(' '), Some('M')) => Ok(StatusIcon::new("○")), // Modified in worktree only
            (Some(' '), Some('D')) => Ok(StatusIcon::new("ｘ")), // Deleted in worktree only
            (Some(' '), Some('T')) => Ok(StatusIcon::new("◇")), // Type changed in worktree only
            (Some(' '), Some('R')) => Ok(StatusIcon::new("↻")), // Renamed in worktree
            (Some(' '), Some('C')) => Ok(StatusIcon::new("⊆")), // Copied in worktree
            (Some(' '), Some('A')) => Ok(StatusIcon::new("⊹")), // Unchanged in index, added in worktree

            (Some('?'), Some('?')) => Ok(StatusIcon::new("⁇")), // Untracked
            (Some('!'), Some('!')) => Ok(StatusIcon::new("")),  // Ignored

            // Default fallback
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}

async fn git_describe_cmd(_opts: &Options) -> Option<SmolStr> {
    git!("describe", "--abbrev=7", "--always", "--tag", "--long")
        .await
        .map(|s| {
            let output = s.trim().split('-').collect::<SmallVec<[_; 4]>>();
            match output[..] {
                [] => "".to_smolstr(),
                [tag] => tag.to_smolstr(),
                [tag, "0"] => tag.to_smolstr(),
                [tag, n] => format_smolstr!("{tag}▴{n}"),
                [tag, "0", hash] => format_smolstr!("{tag}|{hash}"),
                [tag, n, hash, ..] => format_smolstr!("{tag}▴{n}|{hash}"),
            }
        })
}

async fn git_rev_parse(origin: bool) -> Option<SmolStr> {
    git!(
        "rev-parse",
        "--abbrev-ref",
        if origin { "origin/HEAD" } else { "HEAD" }
    )
    .await
    .filter(|s| !s.is_empty())
    .and_then(|s| s.trim().split('/').next_back().map(Into::into))
}

async fn git_name_rev(_opts: &Options) -> Option<SmolStr> {
    let mut result = git!("name-rev", "--name-only", "HEAD").await?;
    for (o, n) in &[
        ("remotes/origin/", "↪"),
        ("remotes/", "↪"),
        ("tags/", ""),
        ("~", "↓"),
    ] {
        result = result.replace_smolstr(o, n);
    }
    Some(result)
}

#[inline]
fn git_bidirectional_inclusion(a: &SmolStr, b: &SmolStr) -> bool {
    a.contains(b.as_str()) || b.contains(a.as_str())
}

#[inline]
async fn git_branch_icon(_: &Options) -> Option<&'static str> {
    let (local, origin) = join!(git_rev_parse(false), git_rev_parse(true));
    match local.as_deref() {
        None => None,
        Some("HEAD") => Some("⚠"),
        Some(local) if Some(local) == origin.as_deref() => Some("⟝"),
        _ => Some("⎇"),
    }
}

#[inline]
async fn git_branch_name(_: &Options) -> Option<SmolStr> {
    git!("branch", "--show")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_smolstr())
}
