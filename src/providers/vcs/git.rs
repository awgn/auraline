use crate::chunk::Chunk;
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::style::to_superscript;
use crate::{cmd::CMD, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, ToSmolStr};
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
        let (icon, info) = join!(git_branch_icon(opts), git_branch_name(opts));
        match (icon, info) {
            (None, None) => None,
            (Some(icon), None) => Some(Chunk::icon(icon)),
            (None, Some(info)) => Some(Chunk::info(info)),
            (Some(icon), Some(info)) => Some(Chunk::new(icon, info)),
        }
    }

    async fn commit(&self, opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        // git describe --always succeeds on any non-empty repo, so run it
        // together with branch_name (already cached) and skip git_name_rev
        // entirely in the common case — it is only needed as a fallback when
        // describe itself fails (e.g. a brand-new empty repo).
        let (branch_name, descr) = join!(git_branch_name(opts), git_describe_cmd(opts));

        if let Some(c) = descr {
            return Some(Chunk::info(c));
        }

        // Fallback: describe failed, try name-rev
        let name_rev = git_name_rev(opts).await;
        match (branch_name, name_rev) {
            (_, None) => None,
            (None, Some(nr)) => Some(Chunk::info(nr)),
            (Some(b), Some(nr)) if git_bidirectional_inclusion(&b, &nr) => None,
            (Some(_), Some(nr)) => Some(Chunk::info(nr)),
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
        let bytes = s.as_bytes();
        if bytes.len() < 2 {
            return Ok(StatusIcon::new(""));
        }
        match (bytes[0], bytes[1]) {
            // Unmerged states (conflicts)
            (b'D', b'D') => Ok(StatusIcon::new("✖")), // Both deleted
            (b'A', b'A') => Ok(StatusIcon::new("⧉")), // Both added
            (b'U', b'U') => Ok(StatusIcon::new("⚠")), // Both modified - warning
            (b'A', b'U') => Ok(StatusIcon::new("⊕")), // Added by us
            (b'U', b'A') => Ok(StatusIcon::new("⊞")), // Added by them
            (b'D', b'U') => Ok(StatusIcon::new("⊖")), // Deleted by us
            (b'U', b'D') => Ok(StatusIcon::new("⊟")), // Deleted by them

            // Index changes
            (b'M', b' ') => Ok(StatusIcon::new("●")), // Modified in index only
            (b'M', b'M') => Ok(StatusIcon::new("◉")), // Modified in both
            (b'M', b'D') => Ok(StatusIcon::new("◐")), // Modified in index, deleted in worktree
            (b'M', b'T') => Ok(StatusIcon::new("◑")), // Modified in index, type changed in worktree

            (b'A', b' ') => Ok(StatusIcon::new("✚")), // Added to index only
            (b'A', b'M') => Ok(StatusIcon::new("✛")), // Added and modified
            (b'A', b'D') => Ok(StatusIcon::new("⊕")), // Added then deleted in worktree
            (b'A', b'T') => Ok(StatusIcon::new("⊛")), // Added, type changed in worktree

            (b'D', b' ') => Ok(StatusIcon::new("−")), // Deleted from index
            (b'D', b'M') => Ok(StatusIcon::new("∓")), // Deleted in index but modified in worktree (weird state)

            (b'R', b' ') => Ok(StatusIcon::new("→")), // Renamed in index
            (b'R', b'M') => Ok(StatusIcon::new("⇢")), // Renamed and modified
            (b'R', b'D') => Ok(StatusIcon::new("⇥")), // Renamed then deleted
            (b'R', b'T') => Ok(StatusIcon::new("⤳")), // Renamed and type changed

            (b'C', b' ') => Ok(StatusIcon::new("⊂")), // Copied in index
            (b'C', b'M') => Ok(StatusIcon::new("⊃")), // Copied and modified
            (b'C', b'D') => Ok(StatusIcon::new("⊄")), // Copied then deleted
            (b'C', b'T') => Ok(StatusIcon::new("⊅")), // Copied and type changed

            (b'T', b' ') => Ok(StatusIcon::new("◈")), // Type changed in index
            (b'T', b'M') => Ok(StatusIcon::new("◊")), // Type changed and modified
            (b'T', b'D') => Ok(StatusIcon::new("⬧")), // Type changed then deleted
            (b'T', b'T') => Ok(StatusIcon::new("⬢")), // Type changed in both

            (b' ', b'M') => Ok(StatusIcon::new("○")), // Modified in worktree only
            (b' ', b'D') => Ok(StatusIcon::new("ｘ")), // Deleted in worktree only
            (b' ', b'T') => Ok(StatusIcon::new("◇")), // Type changed in worktree only
            (b' ', b'R') => Ok(StatusIcon::new("↻")), // Renamed in worktree
            (b' ', b'C') => Ok(StatusIcon::new("⊆")), // Copied in worktree
            (b' ', b'A') => Ok(StatusIcon::new("⊹")), // Unchanged in index, added in worktree

            (b'?', b'?') => Ok(StatusIcon::new("⁇")), // Untracked
            (b'!', b'!') => Ok(StatusIcon::new("")),  // Ignored

            // Default fallback
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}

async fn git_describe_cmd(_opts: &Options) -> Option<SmolStr> {
    git!("describe", "--abbrev=8", "--always", "--tag", "--long")
        .await
        .map(|s| {
            let output = s.trim().split('-').collect::<SmallVec<[_; 4]>>();
            match output[..] {
                [] => "".to_smolstr(),
                [tag] => tag.to_smolstr(),
                [tag, "0"] => tag.to_smolstr(),
                [tag, n] => format_smolstr!("{tag}▴{n}"),
                [tag, "0", hash] => format_smolstr!("{tag}∷{}", &hash[1..]),
                [tag, n, hash, ..] => format_smolstr!("{tag}▴{n}∷{}", &hash[1..]),
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
    let result = git!("name-rev", "--name-only", "HEAD").await?;
    let s = result.as_str();

    let prefix_replaced = if let Some(stripped) = s.strip_prefix("remotes/origin/") {
        format_smolstr!("↪{}", stripped)
    } else if let Some(stripped) = s.strip_prefix("remotes/") {
        format_smolstr!("↪{}", stripped)
    } else if let Some(stripped) = s.strip_prefix("tags/") {
        stripped.to_smolstr()
    } else {
        result
    };

    if prefix_replaced.contains('~') {
        Some(prefix_replaced.replace('~', "↓").into())
    } else {
        Some(prefix_replaced)
    }
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
    // Reuses the cached result of `git rev-parse --abbrev-ref HEAD` that
    // git_branch_icon already requests — no extra subprocess needed.
    git_rev_parse(false)
        .await
        .filter(|s| s != "HEAD" && !s.is_empty())
}
