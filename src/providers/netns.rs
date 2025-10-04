use smol_str::SmolStr;

use crate::{chunk::Chunk, cmd::CMD, options::Options};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.netns {
        return None;
    }
    CMD.exec("ip", ["netns", "identify"])
        .await
        .filter(|s| !s.is_empty())
        .map(|s| Chunk::new(opts.select_str("{}", ""), s))
}
