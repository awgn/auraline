use crate::cmd::CMD;
use crate::providers::vcs::{merge_icons, StatusIcon};
use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder, ToSmolStr};
use std::path::Path;
use std::str::FromStr;
use tokio::fs;
use tokio::io::AsyncReadExt;

macro_rules! hg {
    ( $( $x:expr ),* ) => {
        CMD.exec("hg", [$( $x ),*])
    };
}

struct Hg;

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

pub async fn branch(_: &Options, base: &Path) -> Option<Chunk<SmolStr>> {
    // hg!("branch")
    //     .await
    //     .map(|s| Chunk::new("hg ⎇", s.trim().to_smolstr()))
    let path = base.join(".hg").join("branch");
    let branch = fs::read_to_string(path).await.ok()?;
    Some(Chunk::new("hg ⎇", branch.trim().to_smolstr()))
}

pub async fn status(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    hg!("status").await.map(|status| {
        Chunk::info(merge_icons(
            status
                .lines()
                .map(|line| line.parse::<StatusIcon<Hg>>().unwrap())
                .collect::<Vec<_>>(),
        ))
    })
}

pub async fn stash(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

pub async fn worktree(_: &Options, base: &Path) -> Option<Chunk<SmolStr>> {
    let path = base.join(".hg").join("sharedpath");
    let sharedpath = fs::read_to_string(&path).await.ok()?;
    sharedpath
        .rfind('/')
        .map(|pos| Chunk::new("⌂", sharedpath[..pos].into()))
}

pub async fn commit(_: &Options, base: &Path) -> Option<Chunk<SmolStr>> {
    // hg!("id").await.map(|s| Chunk::info(s.trim().to_smolstr()))
    let hash = get_hg_commit_hash(base).await?;

    let bookmark_path = base.join(".hg").join("bookmarks.current");
    let bookmark = fs::read_to_string(bookmark_path).await.ok();
    match bookmark {
        Some(bm) if !bm.trim().is_empty() => {
            let combined = format_smolstr!("{} ({})", hash.trim(), bm.trim());
            Some(Chunk::info(combined))
        }
        _ => Some(Chunk::info(hash)),
    }

    //Some(Chunk::info(hash))
}

pub async fn divergence(_: &Options, _base: &Path) -> Option<Chunk<SmolStr>> {
    None
}

async fn get_hg_commit_hash(base: &Path) -> Option<SmolStr> {
    // Define the path to the dirstate file.
    let dirstate_path = base.join(".hg").join("dirstate");

    // A Mercurial hash is 20 bytes long (which translates to 40 hex characters).
    let mut hash_bytes = [0u8; 20];

    let mut file = fs::File::open(dirstate_path).await.ok()?;

    // Read exactly 20 bytes from the file into our buffer.
    // `read_exact` ensures that we get all 20 bytes or it returns an error.
    file.read_exact(&mut hash_bytes).await.ok()?;

    let mut hex_string = SmolStrBuilder::new();

    // Iterate over each byte in the buffer and format it as a two-digit
    // hexadecimal string. Dump only the first 8 bytes (16 hex characters).
    for byte in &hash_bytes[..8] {
        hex_string.push_str(&format!("{:02x}", byte));
    }

    hex_string.push('+');

    // Return the resulting hexadecimal string.
    Some(hex_string.finish())
}
