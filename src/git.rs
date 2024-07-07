use itertools::Itertools;
use std::ffi::OsStr;
use tokio::{join, process::Command};

pub async fn git_describe() -> Option<String> {
    let output = git(["describe", "--abbrev=8", "--always", "--tag", "--long"]).await;
    output.map(|s| {
        let output = s.trim().split('-').collect::<Vec<_>>();
        match output[..] {
            [] => "".to_string(),
            [tag] => tag.to_string(),
            [tag, "0"] => tag.to_string(),
            [tag, n] => (tag.to_owned() + "▴" + n).to_string(),
            [tag, "0", hash] => (tag.to_string()) + "|" + hash,
            [tag, n, hash, ..] => (tag.to_owned() + "▴" + n).to_string() + "|" + hash,
        }
    })
}

pub async fn git_branch_name() -> Option<String> {
    match git_branch_show().await {
        None => match git_describe_exact_match().await {
            None => git_rev_parse(true).await,
            x => x,
        },
        x => x,
    }
}

async fn git_branch_show() -> Option<String> {
    git(["branch", "--show"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    })
}

async fn git_describe_exact_match() -> Option<String> {
    git(["describe", "--exact-match"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.replace('~', "↓").trim().to_string())
        }
    })
}

async fn git_rev_parse(origin: bool) -> Option<String> {
    let args = if origin {
        ["rev-parse", "--abbrev-ref", "origin/HEAD"]
    } else {
        ["rev-parse", "--abbrev-ref", "HEAD"]
    };
    git(args).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            s.trim().split('/').last().map(|s| s.to_string())
        }
    })
}

async fn git_name_rev() -> Option<String> {
    let xs = git(["name-rev", "--name-only", "HEAD"]).await?;
    let mut result = xs;
    for (o, n) in &[
        ("tags/", ""),
        ("~", "↓"),
        ("remotes/", "ʀ "),
        ("remotes/origin/", "ᐲ "),
    ] {
        result = result.replace(o, n);
    }
    Some(result)
}

pub async fn git_commit_name(branch_name: Option<String>, descr: Option<String>) -> Option<String> {
    let name_rev = git_name_rev().await?;

    if let Some(branch_name) = branch_name {
        if name_rev.contains(&branch_name) || branch_name.contains(&name_rev) {
            return None;
        }
    }

    if let Some(descr) = descr {
        if name_rev.contains(&descr) || descr.contains(&name_rev){
            return None;
        }
    }

    Some(name_rev)
}

pub async fn git_branch_icon() -> Option<String> {
    let l = git_rev_parse(false).await?;
    if l == "HEAD" {
        Some("⚠ ".into())
    } else {
        let r = git_rev_parse(true).await?;
        if l == r {
            Some("⟝ ".into())
        } else {
            Some("⎇ ".into())
        }
    }
}

pub async fn git_status_icon() -> Option<String> {
    let output = git(["status", "--porcelain"]).await?;
    let icons = output.lines().map(GitIcon::new).collect::<Vec<_>>();
    Some(merge_icons(icons))
}

pub async fn git_stash_counter() -> Option<String> {
    let n = git(["stash", "list"]).await?;
    if n.is_empty() {
        None
    } else {
        let n = &n.lines().count().to_string();
        ("≡".to_string() + &to_superscript(n)).into()
    }
}

pub fn to_superscript(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let super_script_char = |c: char| {
        static STATIC_SUPER_SCRIPT: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
        if !c.is_ascii_digit() {
            return c;
        }
        STATIC_SUPER_SCRIPT[c as usize - '0' as usize]
    };

    for c in s.chars() {
        result.push(super_script_char(c));
    }

    result
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GitIcon(&'static str);

impl GitIcon {
    fn new(input: &str) -> GitIcon {
        match input.chars().collect::<Vec<char>>().as_slice() {
            [' ', 'M', ..] => GitIcon("•"),
            [' ', 'D', ..] => GitIcon("-"),
            [' ', 'A', ..] => GitIcon("+"),
            [' ', 'C', ..] => GitIcon("ᶜ"),
            [' ', 'R', ..] => GitIcon("ᵣ"),
            ['D', 'D', ..] => GitIcon("╌"),
            ['A', 'U', ..] => GitIcon("✛"),
            ['U', 'D', ..] => GitIcon("-"),
            ['U', 'A', ..] => GitIcon("⊕"),
            ['D', 'U', ..] => GitIcon("-"),
            ['A', 'A', ..] => GitIcon("ǂ"),
            ['U', 'U', ..] => GitIcon("☢"),
            ['M', 'D', ..] => GitIcon("✫"),
            ['M', ..] => GitIcon("★"),
            ['T', ..] => GitIcon("¿"),
            ['A', 'D', ..] => GitIcon("∓"),
            ['A', 'M', ..] => GitIcon("∔"),
            ['A', ..] => GitIcon("✛"),
            ['D', 'A', ..] => GitIcon("±"),
            ['D', 'M', ..] => GitIcon("߸"),
            ['D', ..] => GitIcon("-"),
            ['C', ..] => GitIcon("©"),
            ['R', ..] => GitIcon("ʀ"),
            ['!', '!', ..] => GitIcon(""), // Ignored and untracked
            ['?', '?', ..] => GitIcon(""), // Ignored and untracked
            _ => GitIcon(""),
        }
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn merge_icons(mut icons: Vec<GitIcon>) -> String {
    let mut data_grouped: Vec<Vec<GitIcon>> = Vec::new();

    icons.sort();

    for (_, chunk) in &icons.into_iter().filter(|i| !i.is_empty()).chunk_by(|n| *n) {
        data_grouped.push(chunk.collect());
    }

    data_grouped
        .iter()
        .map(|xs| render_icon((xs[0], xs.len())))
        .collect::<Vec<_>>()
        .join("")
}

fn render_icon((icon, n): (GitIcon, usize)) -> String {
    if n == 1 {
        icon.0.to_string()
    } else {
        icon.0.to_string() + &to_superscript(&n.to_string())
    }
}

pub async fn git_ahead_behind_icon() -> Option<String> {
    let (ahead, behind) = join!(tokio::spawn(async { git(["rev-list", "--count", "HEAD@{upstream}..HEAD"]).await } ),
                                tokio::spawn(async { git(["rev-list", "--count", "HEAD@{upstream}..HEAD"]).await } ));

    let ahead = ahead.unwrap_or(None)?;
    let behind = behind.unwrap_or(None)?;

    let is_zero = |x: &String| x.is_empty() || x == "0";

    match (&ahead, &behind) {
        _ if is_zero(&ahead) && is_zero(&behind) => None,
        _ if is_zero(&ahead) => Some(format!("↓{}", behind)),
        _ if is_zero(&behind) => Some(format!("↑{}", ahead)),
        _ => Some(format!("↑{}↓{}", ahead, behind)),
    }
}

pub async fn git<I, S>(args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git").args(args).output().await.ok()?;

    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    if output.status.success() {
        Some(String::from_utf8(output.stdout).unwrap().trim_end().to_string())
    } else {
        None
    }
}
