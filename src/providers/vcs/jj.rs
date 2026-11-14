use crate::cmd::CMD;
use crate::providers::vcs::{merge_icons, StatusIcon, VcsTrait};
use crate::{chunk::Chunk, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr};
use std::path::Path;
use std::str::FromStr;

const JJ_SNAPSHOT_TEMPLATE: &str = r#"bookmarks.map(|b| b.name()).join(",") ++ "\t" ++ working_copies.join(" ") ++ "\t" ++ change_id.short(8) ++ "\t" ++ commit_id.short(8) ++ "\n" ++ diff.summary()"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jj;

struct JjHeader<'a> {
    bookmarks: Option<&'a str>,
    workspace: Option<&'a str>,
    change_id: &'a str,
    commit_id: &'a str,
}

fn parse_jj_header(line: &str) -> Option<JjHeader<'_>> {
    let mut fields = line.split('\t');
    let bookmarks_raw = fields.next()?.trim();
    let wc_raw = fields.next()?.trim();
    let change_id = fields.next()?.trim();
    let commit_id = fields.next()?.trim();

    if change_id.is_empty() || commit_id.is_empty() {
        return None;
    }

    let bookmarks = (!bookmarks_raw.is_empty()).then_some(bookmarks_raw);
    let workspace = wc_raw
        .split_whitespace()
        .next()
        .map(|w| w.trim_end_matches('@'))
        .filter(|w| !w.is_empty() && *w != "default");

    Some(JjHeader {
        bookmarks,
        workspace,
        change_id,
        commit_id,
    })
}

async fn query_jj(path: &Path) -> Option<SmolStr> {
    if let Some(out) = CMD
        .exec_in(
            Some(path),
            "jj",
            [
                "log",
                "--no-graph",
                "--color",
                "never",
                "--no-pager",
                "-r",
                "@",
                "-T",
                JJ_SNAPSHOT_TEMPLATE,
            ],
        )
        .await
    {
        return Some(out);
    }

    // Fallback when working copy cannot be snapshotted (e.g. read-only mount or lock contention).
    CMD.exec_in(
        Some(path),
        "jj",
        [
            "log",
            "--no-graph",
            "--ignore-working-copy",
            "--color",
            "never",
            "--no-pager",
            "-r",
            "@",
            "-T",
            JJ_SNAPSHOT_TEMPLATE,
        ],
    )
    .await
}

impl VcsTrait for Jj {
    async fn branch(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let out = query_jj(path).await?;
        let header = parse_jj_header(out.lines().next()?)?;
        let bm = header.bookmarks?;
        Some(Chunk::new("jj ⎇", SmolStr::new(bm)))
    }

    async fn commit(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let out = query_jj(path).await?;
        let header = parse_jj_header(out.lines().next()?)?;
        Some(Chunk::new(
            "⭑",
            format_smolstr!("{} {}", header.change_id, header.commit_id),
        ))
    }

    async fn status(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let out = query_jj(path).await?;
        let icons = merge_icons(
            out.lines()
                .skip(1)
                .filter(|l| l.as_bytes().get(1) == Some(&b' '))
                .filter_map(|line| line.parse::<StatusIcon<Jj>>().ok())
                .collect::<SmallVec<[_; 8]>>(),
        );
        if icons.is_empty() {
            None
        } else {
            Some(Chunk::info(icons))
        }
    }

    async fn worktree(&self, _opts: &Options, path: &Path) -> Option<Chunk<SmolStr>> {
        let out = query_jj(path).await?;
        let header = parse_jj_header(out.lines().next()?)?;
        header.workspace.map(|ws| Chunk::new("¶", SmolStr::new(ws)))
    }

    async fn stash(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }

    async fn divergence(&self, _opts: &Options, _path: &Path) -> Option<Chunk<SmolStr>> {
        None
    }
}

impl FromStr for StatusIcon<Jj> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.split_whitespace().next();
        match chars {
            Some("A") => Ok(StatusIcon::new("✚")),
            Some("R") => Ok(StatusIcon::new("→")),
            Some("M") => Ok(StatusIcon::new("●")),
            Some("C") => Ok(StatusIcon::new("⊂")),
            Some("D") => Ok(StatusIcon::new("−")),
            _ => Ok(StatusIcon::new("")), // Unknown state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_jj_header;

    #[test]
    fn test_parse_jj_header_structured() {
        let h = parse_jj_header("main\t\tqpvuntsm\t12345678").unwrap();
        assert_eq!(h.bookmarks, Some("main"));
        assert_eq!(h.workspace, None);
        assert_eq!(h.change_id, "qpvuntsm");
        assert_eq!(h.commit_id, "12345678");

        let h_ws = parse_jj_header("🚀,feat\tsecondary@\tqpvuntsm\t12345678").unwrap();
        assert_eq!(h_ws.bookmarks, Some("🚀,feat"));
        assert_eq!(h_ws.workspace, Some("secondary"));

        let h_default_ws = parse_jj_header("\tdefault@\tqpvuntsm\t12345678").unwrap();
        assert_eq!(h_default_ws.bookmarks, None);
        assert_eq!(h_default_ws.workspace, None);
    }
}
