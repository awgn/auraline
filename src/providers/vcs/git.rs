use crate::chunk::{Adjoin, Chunk};
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::style::{to_subscript, to_superscript};
use crate::{cmd::CMD, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, ToSmolStr};
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::str::FromStr;

macro_rules! git {
    ( $( $x:expr ),* ) => {
        CMD.exec("git", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Git {
    git_dir: PathBuf,
    common_dir: PathBuf,
    local_branch: Option<SmolStr>,
    head_hash: Option<SmolStr>,
}

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

fn dir_has_any_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() || (path.is_dir() && dir_has_any_file(&path)) {
            return true;
        }
    }
    false
}

fn resolve_ref_short_hash(common_dir: &Path, ref_name: &str) -> Option<SmolStr> {
    let ref_file = common_dir.join(ref_name);
    if let Ok(content) = std::fs::read_to_string(ref_file) {
        let hash = content.lines().next()?.trim();
        if hash.len() >= 8 {
            return Some(SmolStr::new(&hash[..8]));
        }
    }

    let packed = std::fs::read_to_string(common_dir.join("packed-refs")).ok()?;
    for line in packed.lines() {
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let r = parts.next()?;
        if r == ref_name && hash.len() >= 8 {
            return Some(SmolStr::new(&hash[..8]));
        }
    }
    None
}

impl Git {
    pub fn new(repo_root: &Path) -> Option<Self> {
        let dot_git = repo_root.join(".git");
        let git_dir = if dot_git.is_dir() {
            dot_git
        } else if dot_git.is_file() {
            let content = std::fs::read_to_string(&dot_git).ok()?;
            let line = content.lines().next()?;
            let gitdir_path = Path::new(line.strip_prefix("gitdir: ")?.trim());
            if gitdir_path.is_absolute() {
                gitdir_path.to_path_buf()
            } else {
                repo_root.join(gitdir_path)
            }
        } else {
            return None;
        };

        let common_dir = std::fs::read_to_string(git_dir.join("commondir"))
            .ok()
            .and_then(|content| {
                let rel = content.lines().next()?.trim();
                (!rel.is_empty()).then(|| {
                    let p = Path::new(rel);
                    if p.is_absolute() {
                        p.to_path_buf()
                    } else {
                        git_dir.join(p)
                    }
                })
            })
            .unwrap_or_else(|| git_dir.clone());

        let (local_branch, head_hash) = std::fs::read_to_string(git_dir.join("HEAD"))
            .ok()
            .and_then(|content| {
                let line = content.lines().next()?.trim();
                if let Some(ref_path) = line.strip_prefix("ref: ") {
                    let ref_path = ref_path.trim();
                    let branch = ref_path
                        .strip_prefix("refs/heads/")
                        .map(SmolStr::new)
                        .unwrap_or_else(|| SmolStr::new(ref_path));
                    let hash = resolve_ref_short_hash(&common_dir, ref_path);
                    Some((Some(branch), hash))
                } else {
                    let short = (line.len() >= 8).then(|| SmolStr::new(&line[..8]));
                    Some((Some(SmolStr::new_static("HEAD")), short))
                }
            })
            .unwrap_or((None, None));

        Some(Self {
            git_dir,
            common_dir,
            local_branch,
            head_hash,
        })
    }

    fn get_remote_head(&self) -> Option<SmolStr> {
        let head_file = self.common_dir.join("refs/remotes/origin/HEAD");
        let content = std::fs::read_to_string(head_file).ok()?;
        let line = content.lines().next()?;
        let target = line.strip_prefix("ref: refs/remotes/origin/")?;
        Some(SmolStr::new(target.trim()))
    }

    fn get_stash_count(&self) -> usize {
        let stash_log = self.common_dir.join("logs/refs/stash");
        if let Ok(content) = std::fs::read_to_string(stash_log) {
            content.lines().count()
        } else {
            0
        }
    }

    fn has_tags(&self) -> bool {
        if dir_has_any_file(&self.common_dir.join("refs/tags")) {
            return true;
        }
        if let Ok(packed) = std::fs::read_to_string(self.common_dir.join("packed-refs")) {
            if packed.contains("refs/tags/") {
                return true;
            }
        }
        false
    }

    fn get_action(&self) -> Option<SmolStr> {
        for dir_name in ["rebase-merge", "rebase-apply"] {
            let dir = self.git_dir.join(dir_name);
            if dir.is_dir() {
                let step = std::fs::read_to_string(dir.join("msgnum"))
                    .or_else(|_| std::fs::read_to_string(dir.join("next")))
                    .ok();
                let total = std::fs::read_to_string(dir.join("end"))
                    .or_else(|_| std::fs::read_to_string(dir.join("last")))
                    .ok();
                if let (Some(s), Some(t)) = (step, total) {
                    let s = s.trim();
                    let t = t.trim();
                    if !s.is_empty() && !t.is_empty() {
                        return Some(format_smolstr!(
                            "rebase {}⁄{}",
                            to_superscript(s),
                            to_subscript(t)
                        ));
                    }
                }
                return Some(SmolStr::new_static("rebase"));
            }
        }

        if self.git_dir.join("MERGE_HEAD").is_file() {
            return Some(SmolStr::new_static("merge"));
        }
        if self.git_dir.join("CHERRY_PICK_HEAD").is_file() {
            return Some(SmolStr::new_static("cherry-pick"));
        }
        if self.git_dir.join("REVERT_HEAD").is_file() {
            return Some(SmolStr::new_static("revert"));
        }
        if self.git_dir.join("BISECT_LOG").is_file() {
            return Some(SmolStr::new_static("bisect"));
        }

        None
    }
}

impl VcsTrait for Git {
    async fn branch(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let local = self.local_branch.as_ref()?;
        let icon = if local == "HEAD" {
            Some("⚠")
        } else {
            let remote = self.get_remote_head();
            if Some(local) == remote.as_ref() {
                Some("⟝")
            } else {
                Some("⎇")
            }
        };

        let name = if local == "HEAD" {
            None
        } else {
            Some(local.clone())
        };

        match (icon, name) {
            (None, None) => None,
            (Some(icon), None) => Some(Chunk::icon(icon)),
            (None, Some(name)) => Some(Chunk::info(name)),
            (Some(icon), Some(name)) => Some(Chunk::new(icon, name)),
        }
    }

    async fn action(
        &self,
        _opts: &Options,
        _path: &Path,
    ) -> Option<Chunk<Adjoin<&'static str, SmolStr>>> {
        self.get_action().map(|a| Chunk::info(Adjoin(("⚡", a))))
    }

    async fn commit(&self, opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        if self.has_tags() {
            if let Some(c) = git_describe_cmd(opts).await {
                return Some(Chunk::info(c));
            }
        }

        let is_detached = self.local_branch.as_deref() == Some("HEAD");
        let name_rev = if is_detached {
            git_name_rev(opts)
                .await
                .filter(|nr| nr.as_str() != "undefined")
        } else {
            None
        };

        match (name_rev, self.head_hash.as_ref()) {
            (Some(nr), Some(hash)) => Some(Chunk::info(format_smolstr!("{nr}∷{hash}"))),
            (Some(nr), None) => Some(Chunk::info(nr)),
            (None, Some(hash)) => Some(Chunk::info(format_smolstr!("HEAD∷{hash}"))),
            (None, None) => None,
        }
    }

    async fn status(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        git!("status", "--porcelain", "--branch")
            .await
            .and_then(|s| {
                let icons = s
                    .lines()
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
        if !self.common_dir.join("worktrees").is_dir() {
            return None;
        }

        let output = git!("worktree", "list").await?;
        output.lines().skip(1).find_map(|line| {
            let mut parts = line.split_whitespace();
            let worktree_path = Path::new(parts.next()?);
            let matches = path.starts_with(worktree_path)
                || worktree_path
                    .canonicalize()
                    .map(|canon| path.starts_with(&canon))
                    .unwrap_or(false);
            if matches {
                parts.next()?; // skip the commit hash
                let name = parts.collect::<SmallVec<[_; 8]>>().join(" ");
                Some(Chunk::new("⌂", name.into()))
            } else {
                None
            }
        })
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        let count = self.get_stash_count();
        if count > 0 {
            let mut buffer = itoa::Buffer::new();
            let n = buffer.format(count);
            Some(Chunk::info(format_smolstr!("≡{}", to_superscript(n))))
        } else {
            None
        }
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

fn parse_git_describe(s: &str) -> SmolStr {
    // rsplitn(3, '-') splits from the right, so the elements are in reverse order:
    // [hash, n, tag] for `<tag>-<n>-g<hash>` (preserving any `-` inside `<tag>`).
    let output = s.trim().rsplitn(3, '-').collect::<SmallVec<[_; 3]>>();
    match output[..] {
        [] => "".to_smolstr(),
        [tag] => tag.to_smolstr(),
        ["0", tag] => tag.to_smolstr(),
        [n, tag] => format_smolstr!("{tag}▴{n}"),
        [hash, "0", tag] => {
            let h = hash.strip_prefix('g').unwrap_or(hash);
            format_smolstr!("{tag}∷{h}")
        }
        [hash, n, tag] => {
            let h = hash.strip_prefix('g').unwrap_or(hash);
            format_smolstr!("{tag}▴{n}∷{h}")
        }
        _ => unreachable!(),
    }
}

async fn git_describe_cmd(_opts: &Options) -> Option<SmolStr> {
    git!("describe", "--abbrev=8", "--tags", "--long")
        .await
        .map(|s| parse_git_describe(&s))
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

#[cfg(test)]
mod tests {
    use super::{parse_git_describe, Git};

    #[test]
    fn test_parse_git_describe() {
        assert_eq!(parse_git_describe("b8bad0c9"), "b8bad0c9");
        assert_eq!(parse_git_describe("v0.9.0-0-gb8bad0c9"), "v0.9.0∷b8bad0c9");
        assert_eq!(
            parse_git_describe("v0.9.0-7-gb8bad0c9"),
            "v0.9.0▴7∷b8bad0c9"
        );
        assert_eq!(
            parse_git_describe("v1.0.0-beta.1-3-g12345678"),
            "v1.0.0-beta.1▴3∷12345678"
        );
        assert_eq!(
            parse_git_describe("release-2026-10-0-gabcdef01"),
            "release-2026-10∷abcdef01"
        );
    }

    #[test]
    fn test_git_action_detection() {
        let tmp =
            std::env::temp_dir().join(format!("auraline-test-git-action-{}", std::process::id()));
        let git_dir = tmp.join(".git");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        let git = Git::new(&tmp).unwrap();
        assert_eq!(git.get_action(), None);
        assert!(!git.has_tags());

        // Test tag detection via refs/tags
        let tags_dir = git_dir.join("refs/tags");
        std::fs::create_dir_all(&tags_dir).unwrap();
        std::fs::write(tags_dir.join("v1.0.0"), "123456\n").unwrap();
        assert!(git.has_tags());
        std::fs::remove_dir_all(&tags_dir).unwrap();

        // Test tag detection via packed-refs
        std::fs::write(
            git_dir.join("packed-refs"),
            "# pack-refs with: peeled fully-peeled sorted\n1234567890abcdef refs/tags/v1.0.0\n",
        )
        .unwrap();
        assert!(git.has_tags());
        std::fs::remove_file(git_dir.join("packed-refs")).unwrap();

        // Test merge
        std::fs::write(git_dir.join("MERGE_HEAD"), "123456\n").unwrap();
        assert_eq!(git.get_action().as_deref(), Some("merge"));
        std::fs::remove_file(git_dir.join("MERGE_HEAD")).unwrap();

        // Test rebase with progress
        let rebase_dir = git_dir.join("rebase-merge");
        std::fs::create_dir_all(&rebase_dir).unwrap();
        std::fs::write(rebase_dir.join("msgnum"), "2\n").unwrap();
        std::fs::write(rebase_dir.join("end"), "5\n").unwrap();
        assert_eq!(git.get_action().as_deref(), Some("rebase ²⁄₅"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
