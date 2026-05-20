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

macro_rules! git {
    ( $( $x:expr ),* ) => {
        CMD.exec("git", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Git;

struct ParsedBranchInfo {
    _local: Option<String>,
    _upstream: Option<String>,
    ahead: u32,
    behind: u32,
}

fn parse_branch_line(line: &str) -> Option<ParsedBranchInfo> {
    let line = line.strip_prefix("## ")?.trim();
    if line.starts_with("HEAD (no branch)") {
        return Some(ParsedBranchInfo {
            _local: None,
            _upstream: None,
            ahead: 0,
            behind: 0,
        });
    }

    let (branches_part, div_part) = if let Some(idx) = line.find('[') {
        let (b, d) = line.split_at(idx);
        (b.trim(), Some(d))
    } else {
        (line, None)
    };

    let mut branch_split = branches_part.split("...");
    let local = branch_split.next()?.trim().to_string();
    let upstream = branch_split.next().map(|s| s.trim().to_string());

    let mut ahead = 0;
    let mut behind = 0;

    if let Some(div) = div_part {
        let div = div.trim_matches(|c| c == '[' || c == ']');
        for part in div.split(',') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("ahead ") {
                ahead = val.parse().unwrap_or(0);
            } else if let Some(val) = part.strip_prefix("behind ") {
                behind = val.parse().unwrap_or(0);
            }
        }
    }

    Some(ParsedBranchInfo {
        _local: Some(local),
        _upstream: upstream,
        ahead,
        behind,
    })
}

fn get_local_branch(repo_root: &Path) -> Option<String> {
    let git_dir = repo_root.join(".git");
    let head_file = if git_dir.is_dir() {
        git_dir.join("HEAD")
    } else if git_dir.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_dir) {
            if let Some(line) = content.lines().next() {
                if let Some(gitdir_path) = line.strip_prefix("gitdir: ") {
                    let real_git_dir = Path::new(gitdir_path.trim());
                    let real_git_dir = if real_git_dir.is_absolute() {
                        real_git_dir.to_path_buf()
                    } else {
                        repo_root.join(real_git_dir)
                    };
                    real_git_dir.join("HEAD")
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    let content = std::fs::read_to_string(head_file).ok()?;
    let line = content.lines().next()?;
    if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
        Some(branch.trim().to_string())
    } else {
        Some("HEAD".to_string())
    }
}

fn get_remote_head(repo_root: &Path) -> Option<String> {
    let git_dir = repo_root.join(".git");
    let head_file = if git_dir.is_dir() {
        git_dir.join("refs/remotes/origin/HEAD")
    } else if git_dir.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_dir) {
            if let Some(line) = content.lines().next() {
                if let Some(gitdir_path) = line.strip_prefix("gitdir: ") {
                    let real_git_dir = Path::new(gitdir_path.trim());
                    let real_git_dir = if real_git_dir.is_absolute() {
                        real_git_dir.to_path_buf()
                    } else {
                        repo_root.join(real_git_dir)
                    };
                    real_git_dir.join("refs/remotes/origin/HEAD")
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    let content = std::fs::read_to_string(head_file).ok()?;
    let line = content.lines().next()?;
    let target = line.strip_prefix("ref: refs/remotes/origin/")?;
    Some(target.trim().to_string())
}

fn get_stash_count(repo_root: &Path) -> Option<usize> {
    let git_dir = repo_root.join(".git");
    let real_git_dir = if git_dir.is_dir() {
        Some(git_dir)
    } else if git_dir.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_dir) {
            if let Some(line) = content.lines().next() {
                if let Some(gitdir_path) = line.strip_prefix("gitdir: ") {
                    let real_git_dir = Path::new(gitdir_path.trim());
                    Some(if real_git_dir.is_absolute() {
                        real_git_dir.to_path_buf()
                    } else {
                        repo_root.join(real_git_dir)
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(dir) = real_git_dir {
        let stash_log = dir.join("logs/refs/stash");
        if stash_log.exists() {
            if let Ok(content) = std::fs::read_to_string(stash_log) {
                Some(content.lines().count())
            } else {
                Some(0)
            }
        } else {
            Some(0)
        }
    } else {
        None
    }
}

impl VcsTrait for Git {
    async fn branch(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let local = get_local_branch(path)?;
        let icon = if local == "HEAD" {
            Some("⚠")
        } else {
            let remote = get_remote_head(path);
            if Some(&local) == remote.as_ref() {
                Some("⟝")
            } else {
                Some("⎇")
            }
        };

        let name = if local == "HEAD" {
            None
        } else {
            Some(SmolStr::new(local))
        };

        match (icon, name) {
            (None, None) => None,
            (Some(icon), None) => Some(Chunk::icon(icon)),
            (None, Some(name)) => Some(Chunk::info(name)),
            (Some(icon), Some(name)) => Some(Chunk::new(icon, name)),
        }
    }

    async fn commit(&self, opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let branch_name = get_local_branch(path)
            .filter(|s| s != "HEAD" && !s.is_empty())
            .map(SmolStr::from);

        let descr = git_describe_cmd(opts).await;

        if let Some(c) = descr {
            return Some(Chunk::info(c));
        }

        let name_rev = git_name_rev(opts).await;
        match (branch_name, name_rev) {
            (_, None) => None,
            (None, Some(nr)) => Some(Chunk::info(nr)),
            (Some(b), Some(nr)) if git_bidirectional_inclusion(&b, &nr) => None,
            (Some(_), Some(nr)) => Some(Chunk::info(nr)),
        }
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        git!("status", "--porcelain", "--branch")
            .await
            .and_then(|s| {
                let icons = s.lines()
                    .skip(1)
                    .filter_map(|l| l.parse::<StatusIcon<Git>>().ok())
                    .collect::<SmallVec<[_; 8]>>();
                if icons.is_empty() {
                    None
                } else {
                    Some(Chunk::info(merge_icons(icons)))
                }
            })
    }

    async fn worktree(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let git_dir = path.join(".git");
        let has_worktrees = if git_dir.is_dir() {
            git_dir.join("worktrees").is_dir()
        } else if git_dir.is_file() {
            if let Ok(content) = std::fs::read_to_string(&git_dir) {
                if let Some(line) = content.lines().next() {
                    if let Some(gitdir_path) = line.strip_prefix("gitdir: ") {
                        let real_git_dir = Path::new(gitdir_path.trim());
                        let real_git_dir = if real_git_dir.is_absolute() {
                            real_git_dir.to_path_buf()
                        } else {
                            path.join(real_git_dir)
                        };
                        real_git_dir.join("worktrees").is_dir()
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !has_worktrees {
            return None;
        }

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

    async fn stash(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        if let Some(count) = get_stash_count(path) {
            if count > 0 {
                let mut buffer = itoa::Buffer::new();
                let n = buffer.format(count);
                return Some(Chunk::info(format_smolstr!("≡{}", to_superscript(n))));
            } else {
                return None;
            }
        }

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
        let status_out = git!("status", "--porcelain", "--branch").await?;
        let first_line = status_out.lines().next()?;
        let info = parse_branch_line(first_line)?;

        match (info.ahead, info.behind) {
            (0, 0) => None,
            (0, behind) => Some(Chunk::info(format_smolstr!("↓{}", behind))),
            (ahead, 0) => Some(Chunk::info(format_smolstr!("↑{}", ahead))),
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
