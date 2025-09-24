use crate::chunk::Chunk;
use crate::style::to_superscript;
use crate::{cmd::CMD, options::Options};
use itertools::Itertools;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder, StrExt, ToSmolStr};
use std::env;
use tokio::join;

macro_rules! git {
    ( $( $x:expr ),* ) => {
        CMD.exec("git", [$( $x ),*])
    };
}

pub async fn describe(opts: &Options) -> Option<Chunk<SmolStr>> {
    git_describe_cmd(opts).await.map(Chunk::info)
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

async fn git_branch_name(_: &Options) -> Option<SmolStr> {
    let (branch, describe_exact_match, rev_parse) = join!(
        git_branch_show(),
        git_describe_exact_match(),
        git_rev_parse(true)
    );

    branch.or(describe_exact_match).or(rev_parse)
}

pub async fn branch(opts: &Options) -> Option<Chunk<SmolStr>> {
    let icon = git_branch_icon(opts).await;
    let info = git_branch_name(opts).await;
    match (icon, info) {
        (None, None) => None,
        (Some(icon), None) => Some(Chunk::icon(icon)),
        (None, Some(info)) => Some(Chunk::info(info)),
        (Some(icon), Some(info)) => Some(Chunk::new(icon, info)),
    }
}

async fn git_describe_exact_match() -> Option<SmolStr> {
    git!("describe", "--exact-match")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_smolstr().replace_smolstr("~", "↓"))
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

pub async fn commit(opts: &Options) -> Option<Chunk<SmolStr>> {
    let (name_rev, branch_name, descr) = join!(
        git_name_rev(opts),
        git_branch_name(opts),
        git_describe_cmd(opts)
    );

    let name_rev = name_rev?;

    if let Some(branch_name) = branch_name {
        if name_rev.contains(branch_name.as_str()) || branch_name.contains(name_rev.as_str()) {
            return None;
        }
    }

    if let Some(descr) = descr {
        if name_rev.contains(descr.as_str()) || descr.contains(name_rev.as_str()) {
            return None;
        }
    }

    Some(Chunk::info(name_rev))
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

pub async fn status(_: &Options) -> Option<Chunk<SmolStr>> {
    git!("status", "--porcelain")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| Chunk::info(merge_icons(s.lines().map(GitIcon::new).collect::<Vec<_>>())))
}

pub async fn worktree(_: &Options) -> Option<Chunk<SmolStr>> {
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

pub async fn stash(_: &Options) -> Option<Chunk<SmolStr>> {
    git!("stash", "list")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut buffer = itoa::Buffer::new();
            let n = buffer.format(s.lines().count());
            Chunk::info(format_smolstr!("≡{}", to_superscript(n)))
        })
}

pub async fn divergence(_: &Options) -> Option<Chunk<SmolStr>> {
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GitIcon(&'static str);

impl GitIcon {
    fn new(input: &str) -> GitIcon {
        let mut chars = input.chars();
        match (chars.next(), chars.next()) {
            // Unmerged states (conflicts)
            (Some('D'), Some('D')) => GitIcon("✖"), // Both deleted
            (Some('A'), Some('A')) => GitIcon("⧉"), // Both added
            (Some('U'), Some('U')) => GitIcon("⚠"), // Both modified - warning
            (Some('A'), Some('U')) => GitIcon("⊕"), // Added by us
            (Some('U'), Some('A')) => GitIcon("⊞"), // Added by them
            (Some('D'), Some('U')) => GitIcon("⊖"), // Deleted by us
            (Some('U'), Some('D')) => GitIcon("⊟"), // Deleted by them

            // Index changes
            (Some('M'), Some(' ')) => GitIcon("●"), // Modified in index only
            (Some('M'), Some('M')) => GitIcon("◉"), // Modified in both
            (Some('M'), Some('D')) => GitIcon("◐"), // Modified in index, deleted in worktree
            (Some('M'), Some('T')) => GitIcon("◑"), // Modified in index, type changed in worktree

            (Some('A'), Some(' ')) => GitIcon("✚"), // Added to index only
            (Some('A'), Some('M')) => GitIcon("✛"), // Added and modified
            (Some('A'), Some('D')) => GitIcon("⊕"), // Added then deleted in worktree
            (Some('A'), Some('T')) => GitIcon("⊛"), // Added, type changed in worktree

            (Some('D'), Some(' ')) => GitIcon("−"), // Deleted from index
            (Some('D'), Some('M')) => GitIcon("∓"), // Deleted in index but modified in worktree (weird state)

            (Some('R'), Some(' ')) => GitIcon("→"), // Renamed in index
            (Some('R'), Some('M')) => GitIcon("⇢"), // Renamed and modified
            (Some('R'), Some('D')) => GitIcon("⇥"), // Renamed then deleted
            (Some('R'), Some('T')) => GitIcon("⤳"), // Renamed and type changed

            (Some('C'), Some(' ')) => GitIcon("⊂"), // Copied in index
            (Some('C'), Some('M')) => GitIcon("⊃"), // Copied and modified
            (Some('C'), Some('D')) => GitIcon("⊄"), // Copied then deleted
            (Some('C'), Some('T')) => GitIcon("⊅"), // Copied and type changed

            (Some('T'), Some(' ')) => GitIcon("◈"), // Type changed in index
            (Some('T'), Some('M')) => GitIcon("◊"), // Type changed and modified
            (Some('T'), Some('D')) => GitIcon("⬧"), // Type changed then deleted
            (Some('T'), Some('T')) => GitIcon("⬢"), // Type changed in both

            (Some(' '), Some('M')) => GitIcon("○"), // Modified in worktree only
            (Some(' '), Some('D')) => GitIcon(""), // Deleted in worktree only
            (Some(' '), Some('T')) => GitIcon("◇"), // Type changed in worktree only
            (Some(' '), Some('R')) => GitIcon("↻"), // Renamed in worktree
            (Some(' '), Some('C')) => GitIcon("⊆"), // Copied in worktree
            (Some(' '), Some('A')) => GitIcon("⊹"), // Unchanged in index, added in worktree

            (Some('?'), Some('?')) => GitIcon("⁇"), // Untracked
            (Some('!'), Some('!')) => GitIcon(""),  // Ignored

            // Default fallback
            _ => GitIcon(""), // Unknown state
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn merge_icons(icons: Vec<GitIcon>) -> SmolStr {
    let mut builder = SmolStrBuilder::new();
    icons
        .into_iter()
        .filter(|i| !i.is_empty()) // Directly filter out empty icons
        .sorted()
        .chunk_by(|icon| *icon) // Efficiently group by GitIcon value using chunk_by
        .into_iter()
        .for_each(|(icon, group)| builder.push_str(&render_icon((icon, group.count()))));

    builder.finish()
}

fn render_icon((icon, n): (GitIcon, usize)) -> SmolStr {
    let mut builder = SmolStrBuilder::new();
    if n == 1 {
        builder.push_str(icon.0);
    } else {
        let mut buffer = itoa::Buffer::new();
        let numb = buffer.format(n);
        builder.push_str(icon.0);
        builder.push_str(&to_superscript(numb));
    }
    builder.finish()
}

async fn git_branch_show() -> Option<SmolStr> {
    git!("branch", "--show")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_smolstr())
}
