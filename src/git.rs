use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{Arc, Mutex},
    task::Poll,
};
use tokio::{join, process::Command};

lazy_static! {
    static ref GIT: GitCache = GitCache::new();
}

#[derive(Debug, Clone)]
struct GitValue(Arc<tokio::sync::Mutex<Poll<Option<String>>>>);

struct GitCache {
    cache: Mutex<HashMap<String, GitValue>>,
}

impl GitCache {
    fn new() -> Self {
        Self {
            cache: Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn make_key<I, S>(args: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        args.into_iter()
            .map(|s| s.as_ref().to_os_string().into_string().unwrap())
            .join(" ")
    }

    pub async fn exec<I, S>(&self, args: I) -> Option<String>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let key = Self::make_key(args.clone());
        let value = {
            let mut cache = self.cache.lock().unwrap();
            let value = cache
                .entry(key)
                .or_insert_with(|| GitValue(Arc::new(tokio::sync::Mutex::new(Poll::Pending))));
            value.clone()
        };

        let mut value = value.0.lock().await;
        match *value {
            Poll::Ready(ref v) => v.clone(),
            Poll::Pending => {
                let output = Self::git(args).await;
                *value = Poll::Ready(output.clone());
                output
            }
        }
    }

    async fn git<I, S>(args: I) -> Option<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git").args(args).output().await.ok()?;

        // tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        if output.status.success() {
            Some(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .trim_end()
                    .to_string(),
            )
        } else {
            None
        }
    }
}

pub async fn git_describe() -> Option<String> {
    let output = GIT
        .exec(["describe", "--abbrev=8", "--always", "--tag", "--long"])
        .await;

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
    let (branch, describe_exact_match, rev_parse) = join!(
        git_branch_show(),
        git_describe_exact_match(),
        git_rev_parse(true)
    );
    match branch {
        None => match describe_exact_match {
            None => rev_parse,
            x => x,
        },
        x => x,
    }
}

pub async fn git_branch_show() -> Option<String> {
    GIT.exec(["branch", "--show"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    })
}

pub async fn git_describe_exact_match() -> Option<String> {
    GIT.exec(["describe", "--exact-match"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.replace('~', "↓").trim().to_string())
        }
    })
}

pub async fn git_rev_parse(origin: bool) -> Option<String> {
    let args = if origin {
        ["rev-parse", "--abbrev-ref", "origin/HEAD"]
    } else {
        ["rev-parse", "--abbrev-ref", "HEAD"]
    };
    GIT.exec(args).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            s.trim().split('/').last().map(|s| s.to_string())
        }
    })
}

pub async fn git_name_rev() -> Option<String> {
    let xs = GIT.exec(["name-rev", "--name-only", "HEAD"]).await?;
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

pub async fn git_commit_name() -> Option<String> {
    let (name_rev, branch_name, descr) = join!(git_name_rev(), git_branch_name(), git_describe());
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

pub async fn git_branch_icon() -> Option<String> {
    let (l, r) = join!(git_rev_parse(false), git_rev_parse(true));
    if l == Some("HEAD".into()) {
        Some("⚠ ".into())
    } else if l == r {
        Some("⟝ ".into())
    } else {
        Some("⎇ ".into())
    }
}

pub async fn git_status_icon() -> Option<String> {
    let output = GIT.exec(["status", "--porcelain"]).await?;
    if output.is_empty() {
        None
    } else {
        let icons = output.lines().map(GitIcon::new).collect::<Vec<_>>();
        Some(merge_icons(icons))
    }
}

pub async fn git_stash_counter() -> Option<String> {
    let n = GIT.exec(["stash", "list"]).await?;
    if n.is_empty() {
        None
    } else {
        let n = &n.lines().count().to_string();
        ("≡".to_string() + &to_superscript(n)).into()
    }
}

pub async fn git_ahead_behind_icon() -> Option<String> {
    let (ahead, behind) = join!(
        GIT.exec(["rev-list", "--count", "HEAD@{upstream}..HEAD"]),
        GIT.exec(["rev-list", "--count", "HEAD@{upstream}..HEAD"])
    );

    let ahead = ahead?;
    let behind = behind?;

    let is_zero = |x: &String| x.is_empty() || x == "0";

    match (&ahead, &behind) {
        _ if is_zero(&ahead) && is_zero(&behind) => None,
        _ if is_zero(&ahead) => Some(format!("↓{}", behind)),
        _ if is_zero(&behind) => Some(format!("↑{}", ahead)),
        _ => Some(format!("↑{}↓{}", ahead, behind)),
    }
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

fn to_superscript(s: &str) -> String {
    const SUPERSCRIPT_DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    s.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                SUPERSCRIPT_DIGITS[c as usize - '0' as usize] // Assuming ASCII
            } else {
                c
            }
        })
        .collect()
}

fn merge_icons(icons: Vec<GitIcon>) -> String {
    icons
        .into_iter()
        .filter(|i| !i.is_empty()) // Directly filter out empty icons
        .chunk_by(|icon| *icon) // Efficiently group by GitIcon value using chunk_by
        .into_iter()
        .map(|(icon, group)| render_icon((icon, group.count())))
        .collect()
}

fn render_icon((icon, n): (GitIcon, usize)) -> String {
    if n == 1 {
        icon.0.to_string()
    } else {
        format!("{}{}", icon.0, to_superscript(&n.to_string()))
    }
}
