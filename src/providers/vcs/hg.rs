use crate::cmd::CMD;
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::{chunk::Chunk, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder, ToSmolStr};
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

macro_rules! hg_in {
    ( $path:expr, $( $x:expr ),* ) => {
        CMD.exec_in(Some($path), "hg", [$( $x ),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hg;

impl VcsTrait for Hg {
    async fn branch(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let path = path.join(".hg").join("branch");
        let branch = std::fs::read_to_string(path).ok()?;
        Some(Chunk::new("hg ⎇", branch.trim().to_smolstr()))
    }

    async fn commit(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let hash = get_hg_commit_hash(path)?;

        let bookmark_path = path.join(".hg").join("bookmarks.current");
        let bookmark = std::fs::read_to_string(bookmark_path).ok();
        match bookmark {
            Some(bm) if !bm.trim().is_empty() => {
                let combined = format_smolstr!("{} ({})", hash.trim(), bm.trim());
                Some(Chunk::info(combined))
            }
            _ => Some(Chunk::info(hash)),
        }
    }

    async fn status(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        hg_in!(path, "status").await.and_then(|status| {
            let icons = merge_icons(
                status
                    .lines()
                    .filter_map(|line| line.parse::<StatusIcon<Hg>>().ok())
                    .collect::<SmallVec<[_; 8]>>(),
            );
            (!icons.is_empty()).then(|| Chunk::info(icons))
        })
    }

    async fn worktree(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let path = path.join(".hg").join("sharedpath");
        let sharedpath = std::fs::read_to_string(path).ok()?;
        sharedpath
            .rfind('/')
            .map(|pos| Chunk::new("⌂", sharedpath[..pos].into()))
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }
}

impl FromStr for StatusIcon<Hg> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next() {
            Some('A') => Ok(StatusIcon::new("✚")), // added
            Some('M') => Ok(StatusIcon::new("●")), // modified
            Some('R') => Ok(StatusIcon::new("✖")), // removed
            Some('!') => Ok(StatusIcon::new("!")), // missing
            Some('?') => Ok(StatusIcon::new("?")), // not tracked
            Some('C') => Ok(StatusIcon::new("")),  // clean
            Some('I') => Ok(StatusIcon::new("")),  // Ignored
            _ => Ok(StatusIcon::new("")),          // Unknown state
        }
    }
}

fn get_hg_commit_hash(base: &Path) -> Option<SmolStr> {
    let dirstate_path = base.join(".hg").join("dirstate");

    // A Mercurial hash is 20 bytes long (40 hex characters).
    let mut hash_bytes = [0u8; 20];
    let mut file = std::fs::File::open(dirstate_path).ok()?;
    file.read_exact(&mut hash_bytes).ok()?;

    let mut hex_string = SmolStrBuilder::new();
    use std::fmt::Write;
    for byte in &hash_bytes[..8] {
        let _ = write!(hex_string, "{:02x}", byte);
    }
    hex_string.push('+');

    Some(hex_string.finish())
}
