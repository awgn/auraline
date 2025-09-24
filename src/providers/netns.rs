use smol_str::SmolStr;

use crate::{chunk::Chunk, cmd::CMD, options::Options};

pub async fn namespace(_: &Options) -> Option<Chunk<SmolStr>> {
    CMD.exec("ip", ["netns", "identify"])
        .await
        .filter(|s| !s.is_empty())
        .map(|s| Chunk::new("", s))
}
