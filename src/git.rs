use itertools::Itertools;
use tokio::join;

use crate::cmd::CMD;

pub async fn git_describe() -> Option<String> {
    let output = CMD
        .exec(
            "git",
            ["describe", "--abbrev=8", "--always", "--tag", "--long"],
        )
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
    CMD.exec("git", ["branch", "--show"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    })
}

pub async fn git_describe_exact_match() -> Option<String> {
    CMD.exec("git", ["describe", "--exact-match"])
        .await
        .and_then(|s| {
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
    CMD.exec("git", args).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            s.trim().split('/').last().map(|s| s.to_string())
        }
    })
}

pub async fn git_name_rev() -> Option<String> {
    let xs = CMD.exec("git", ["name-rev", "--name-only", "HEAD"]).await?;
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
    let (local, origin) = join!(git_rev_parse(false), git_rev_parse(true));

    match (&local, &origin) {
        (None, _) => None,
        (Some(ref local), _) if local == "HEAD" => Some("⚠ ".into()),
        _ => {
            if local == origin {
                Some("⟝ ".into())
            } else {
                Some("⎇ ".into())
            }
        }
    }
}

pub async fn git_status_icon() -> Option<String> {
    let output = CMD.exec("git", ["status", "--porcelain"]).await?;
    if output.is_empty() {
        None
    } else {
        let icons = output.lines().map(GitIcon::new).collect::<Vec<_>>();
        Some(merge_icons(icons))
    }
}

pub async fn git_stash_counter() -> Option<String> {
    let n = CMD.exec("git", ["stash", "list"]).await?;
    if n.is_empty() {
        None
    } else {
        let n = &n.lines().count().to_string();
        ("≡".to_string() + &to_superscript(n)).into()
    }
}

pub async fn git_ahead_behind_icon() -> Option<String> {
    let (ahead, behind) = join!(
        CMD.exec("git", ["rev-list", "--count", "HEAD@{upstream}..HEAD"]),
        CMD.exec("git", ["rev-list", "--count", "HEAD@{upstream}..HEAD"])
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
