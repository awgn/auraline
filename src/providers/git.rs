use crate::{cmd::CMD, options::Options};
use itertools::Itertools;
use std::{borrow::Cow, env};
use tokio::join;
use crate::style::to_superscript;

macro_rules! git {
    ( $( $x:expr ),* ) => {
        CMD.exec("git", [$( $x ),*])
    };
}

#[inline]
pub async fn git_describe(opts: &Options) -> Option<String> {
    if opts.fast {
        git_describe_fast().await
    } else {
        git_describe_slow().await
    }
}

pub async fn git_describe_slow() -> Option<String> {
    git!("describe", "--abbrev=7", "--always", "--tag", "--long")
        .await
        .map(|s| {
            let output = s.trim().split('-').collect::<Vec<_>>();
            match output[..] {
                [] => "".to_owned(),
                [tag] => tag.to_owned(),
                [tag, "0"] => tag.to_owned(),
                [tag, n] => tag.to_owned() + "▴" + n,
                [tag, "0", hash] => tag.to_owned() + "|" + hash,
                [tag, n, hash, ..] => tag.to_owned() + "▴" + n + "|" + hash,
            }
        })
}

pub async fn git_describe_fast() -> Option<String> {
    git!("rev-parse", "--short", "HEAD")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
}

pub async fn git_branch_name(_:&Options) -> Option<String> {
    let (branch, describe_exact_match, rev_parse) = join!(
        git_branch_show(),
        git_describe_exact_match(),
        git_rev_parse(true)
    );

    branch.or(describe_exact_match).or(rev_parse)
}

pub async fn git_branch_show() -> Option<String> {
    git!("branch", "--show")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
}

pub async fn git_describe_exact_match() -> Option<String> {
    git!("describe", "--exact-match")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('~', "↓").trim().to_string())
}

pub async fn git_rev_parse(origin: bool) -> Option<String> {
    git!(
        "rev-parse",
        "--abbrev-ref",
        if origin { "origin/HEAD" } else { "HEAD" }
    )
    .await
    .filter(|s| !s.is_empty())
    .and_then(|s| s.trim().split('/').next_back().map(|s| s.to_string()))
}

pub async fn git_name_rev(fast: bool) -> Option<String> {
    if fast {
        return None;
    }
    let mut result = git!("name-rev", "--name-only", "HEAD").await?;
    for (o, n) in &[
        ("remotes/origin/", "ᐲ•"),
        ("remotes/", "⟢•"),
        ("tags/", ""),
        ("~", "↓"),
    ] {
        result = result.replace(o, n);
    }
    Some(result)
}

pub async fn git_commit_name(opts: &Options) -> Option<String> {
    let (name_rev, branch_name, descr) =
        join!(git_name_rev(opts.fast), git_branch_name(opts), git_describe(opts));

    let name_rev = name_rev?;

    if let Some(branch_name) = branch_name {
        if name_rev.contains(&branch_name) || branch_name.contains(&name_rev) {
            return None;
        }
    }

    if let Some(descr) = descr {
        if name_rev.contains(&descr) || descr.contains(&name_rev) {
            return None;
        }
    }

    Some(name_rev)
}

pub async fn git_branch_icon(_: &Options) -> Option<&'static str> {
    let (local, origin) = join!(git_rev_parse(false), git_rev_parse(true));
    match local.as_deref() {
        None => None,
        Some("HEAD") => Some("⚠"),
        Some(local) if Some(local) == origin.as_deref() => Some("⟝"),
        _ => Some("⎇"),
    }
}

pub async fn git_status_icon(_: &Options) -> Option<String> {
    git!("status", "--porcelain")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| merge_icons(s.lines().map(GitIcon::new).collect::<Vec<_>>()))
}

pub async fn git_worktree(_:&Options) -> Option<String> {
    let path = env::current_dir().ok()?;
    let output = git!("worktree", "list").await?;
    output.lines().skip(1).find_map(|line| {
        let mut parts = line.split_whitespace();
        let worktree_path = parts.next()?;
        if path.starts_with(worktree_path) {
            parts.next()?; // skip the branch
            let name = parts.collect::<Vec<_>>().join(" ");
            Some(format!("⌂{}", name))
        } else {
            None
        }
    })
}

pub async fn git_stash_counter(_ :&Options) -> Option<String> {
    git!("stash", "list")
        .await
        .filter(|s| !s.is_empty())
        .map(|s| {
            let n = &s.lines().count().to_string();
            format!("≡{}",to_superscript(n))
        })
}

pub async fn git_ahead_behind_icon(_: &Options) -> Option<String> {
    let (ahead, behind) = join!(
        git!("rev-list", "--count", "HEAD@{upstream}..HEAD"),
        git!("rev-list", "--count", "HEAD..HEAD@{upstream}")
    );

    let ahead = ahead?;
    let behind = behind?;

    match (ahead.as_str(), behind.as_str()) {
        ("0" | "", "0" | "") => None,
        ("0" | "", behind) => Some(format!("↓{}", behind)),
        (ahead, "0" | "") => Some(format!("↑{}", ahead)),
        (ahead, behind) => Some(format!("↑{}↓{}", ahead, behind)),
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GitIcon(&'static str);

impl GitIcon {
    fn new(input: &str) -> GitIcon {
        let mut chars = input.chars();
        match (chars.next(), chars.next()) {
            (Some(' '), Some('M')) => GitIcon("•"),
            (Some(' '), Some('D')) => GitIcon("-"),
            (Some(' '), Some('A')) => GitIcon("+"),
            (Some(' '), Some('C')) => GitIcon("ᶜ"),
            (Some(' '), Some('R')) => GitIcon("ᵣ"),
            (Some('D'), Some('D')) => GitIcon("╌"),
            (Some('A'), Some('U')) => GitIcon("✛"),
            (Some('U'), Some('D')) => GitIcon("-"),
            (Some('U'), Some('A')) => GitIcon("⊕"),
            (Some('D'), Some('U')) => GitIcon("-"),
            (Some('A'), Some('A')) => GitIcon("ǂ"),
            (Some('U'), Some('U')) => GitIcon("☢"),
            (Some('M'), Some('D')) => GitIcon("✫"),
            (Some('M'), _) => GitIcon("★"),
            (Some('T'), _) => GitIcon("¿"),
            (Some('A'), Some('D')) => GitIcon("∓"),
            (Some('A'), Some('M')) => GitIcon("∔"),
            (Some('A'), _) => GitIcon("✛"),
            (Some('D'), Some('A')) => GitIcon("±"),
            (Some('D'), Some('M')) => GitIcon("߸"),
            (Some('D'), _) => GitIcon("-"),
            (Some('C'), _) => GitIcon("©"),
            (Some('R'), _) => GitIcon("ʀ"),
            (Some('!'), Some('!')) => GitIcon(""), // Ignored and untracked
            (Some('?'), Some('?')) => GitIcon(""), // Ignored and untracked
            _ => GitIcon(""),
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[inline]
fn merge_icons(icons: Vec<GitIcon>) -> String {
    icons
        .into_iter()
        .filter(|i| !i.is_empty()) // Directly filter out empty icons
        .chunk_by(|icon| *icon) // Efficiently group by GitIcon value using chunk_by
        .into_iter()
        .map(|(icon, group)| render_icon((icon, group.count())))
        .collect()
}

#[inline]
fn render_icon((icon, n): (GitIcon, usize)) -> Cow<'static, str> {
    if n == 1 {
        Cow::Borrowed(icon.0)
    } else {
        let mut buffer = itoa::Buffer::new();
        let numb = buffer.format(n);
        let mut concatenated = String::with_capacity(icon.0.len() + numb.len());
        concatenated.push_str(icon.0);
        concatenated.push_str(&to_superscript(numb));
        Cow::Owned(concatenated)
    }
}
