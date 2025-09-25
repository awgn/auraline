use crate::chunk::Chunk;
use crate::providers::vcs::merge_icons;
use crate::style::to_superscript;
use crate::{cmd::CMD, options::Options};
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct StatusIcon(pub &'static str);

impl AsRef<str> for StatusIcon {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl FromStr for StatusIcon {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            // Unmerged states (conflicts)
            (Some('D'), Some('D')) => Ok(StatusIcon("✖")), // Both deleted
            (Some('A'), Some('A')) => Ok(StatusIcon("⧉")), // Both added
            (Some('U'), Some('U')) => Ok(StatusIcon("⚠")), // Both modified - warning
            (Some('A'), Some('U')) => Ok(StatusIcon("⊕")), // Added by us
            (Some('U'), Some('A')) => Ok(StatusIcon("⊞")), // Added by them
            (Some('D'), Some('U')) => Ok(StatusIcon("⊖")), // Deleted by us
            (Some('U'), Some('D')) => Ok(StatusIcon("⊟")), // Deleted by them

            // Index changes
            (Some('M'), Some(' ')) => Ok(StatusIcon("●")), // Modified in index only
            (Some('M'), Some('M')) => Ok(StatusIcon("◉")), // Modified in both
            (Some('M'), Some('D')) => Ok(StatusIcon("◐")), // Modified in index, deleted in worktree
            (Some('M'), Some('T')) => Ok(StatusIcon("◑")), // Modified in index, type changed in worktree

            (Some('A'), Some(' ')) => Ok(StatusIcon("✚")), // Added to index only
            (Some('A'), Some('M')) => Ok(StatusIcon("✛")), // Added and modified
            (Some('A'), Some('D')) => Ok(StatusIcon("⊕")), // Added then deleted in worktree
            (Some('A'), Some('T')) => Ok(StatusIcon("⊛")), // Added, type changed in worktree

            (Some('D'), Some(' ')) => Ok(StatusIcon("−")), // Deleted from index
            (Some('D'), Some('M')) => Ok(StatusIcon("∓")), // Deleted in index but modified in worktree (weird state)

            (Some('R'), Some(' ')) => Ok(StatusIcon("→")), // Renamed in index
            (Some('R'), Some('M')) => Ok(StatusIcon("⇢")), // Renamed and modified
            (Some('R'), Some('D')) => Ok(StatusIcon("⇥")), // Renamed then deleted
            (Some('R'), Some('T')) => Ok(StatusIcon("⤳")), // Renamed and type changed

            (Some('C'), Some(' ')) => Ok(StatusIcon("⊂")), // Copied in index
            (Some('C'), Some('M')) => Ok(StatusIcon("⊃")), // Copied and modified
            (Some('C'), Some('D')) => Ok(StatusIcon("⊄")), // Copied then deleted
            (Some('C'), Some('T')) => Ok(StatusIcon("⊅")), // Copied and type changed

            (Some('T'), Some(' ')) => Ok(StatusIcon("◈")), // Type changed in index
            (Some('T'), Some('M')) => Ok(StatusIcon("◊")), // Type changed and modified
            (Some('T'), Some('D')) => Ok(StatusIcon("⬧")), // Type changed then deleted
            (Some('T'), Some('T')) => Ok(StatusIcon("⬢")), // Type changed in both

            (Some(' '), Some('M')) => Ok(StatusIcon("○")), // Modified in worktree only
            (Some(' '), Some('D')) => Ok(StatusIcon("")), // Deleted in worktree only
            (Some(' '), Some('T')) => Ok(StatusIcon("◇")), // Type changed in worktree only
            (Some(' '), Some('R')) => Ok(StatusIcon("↻")), // Renamed in worktree
            (Some(' '), Some('C')) => Ok(StatusIcon("⊆")), // Copied in worktree
            (Some(' '), Some('A')) => Ok(StatusIcon("⊹")), // Unchanged in index, added in worktree

            (Some('?'), Some('?')) => Ok(StatusIcon("⁇")), // Untracked
            (Some('!'), Some('!')) => Ok(StatusIcon("")),  // Ignored

            // Default fallback
            _ => Ok(StatusIcon("")), // Unknown state
        }
    }
}

async fn git_describe_cmd(_opts: &Options) -> Option<SmolStr> {
    git!("describe", "--abbrev=7", "--always", "--tag", "--long")
        .await
        .map(|s| {
            let output = s.trim().split('-').collect::<Vec<_>>();
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

pub async fn branch(opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
    let icon = git_branch_icon(opts).await;
    let info = git_branch_name(opts).await;
    match (icon, info) {
        (None, None) => None,
        (Some(icon), None) => Some(Chunk::icon(icon)),
        (None, Some(info)) => Some(Chunk::info(info)),
        (Some(icon), Some(info)) => Some(Chunk::new(icon, info)),
    }
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
fn bidirectional_inclusion(a: &SmolStr, b: &SmolStr) -> bool {
    a.contains(b.as_str()) || b.contains(a.as_str())
}

pub async fn commit(opts: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    let (name_rev, branch_name, descr) = join!(
        git_name_rev(opts),
        git_branch_name(opts),
        git_describe_cmd(opts)
    );

    match (branch_name, descr, name_rev) {
        (_, None, None) => None,
        (None, None, Some(nr)) => Some(Chunk::info(nr)),
        (Some(b), None, Some(nr)) if bidirectional_inclusion(&b, &nr) => None,
        (Some(_), None, Some(nr)) => Some(Chunk::info(nr)),
        (_, Some(c), _) => Some(Chunk::info(c)),
    }
}

async fn git_branch_icon(_: &Options) -> Option<&'static str> {
    let (local, origin) = join!(git_rev_parse(false), git_rev_parse(true));
    match local.as_deref() {
        None => None,
        Some("HEAD") => Some("⚠"),
        Some(local) if Some(local) == origin.as_deref() => Some("⟝"),
        _ => Some("⎇"),
    }
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    git!("status", "--porcelain")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| {
            Chunk::info(merge_icons(
                s.lines()
                    .map(|l| l.parse::<StatusIcon>().unwrap())
                    .collect::<Vec<_>>(),
            ))
        })
}

pub async fn worktree(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    let path = env::current_dir().ok()?;
    let output = git!("worktree", "list").await?;
    output.lines().skip(1).find_map(|line| {
        let mut parts = line.split_whitespace();
        let worktree_path = parts.next()?;
        if path.starts_with(worktree_path) {
            parts.next()?; // skip the branch
            let name = parts.collect::<Vec<_>>().join(" ");
            Some(Chunk::new("⌂", name.into()))
        } else {
            None
        }
    })
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    git!("stash", "list")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut buffer = itoa::Buffer::new();
            let n = buffer.format(s.lines().count());
            Chunk::info(format_smolstr!("≡{}", to_superscript(n)))
        })
}

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
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

async fn git_branch_name(_: &Options) -> Option<SmolStr> {
    git!("branch", "--show")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_smolstr())
}
