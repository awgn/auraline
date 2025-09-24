use smol_str::{format_smolstr, SmolStr};

use crate::{chunk::Chunk, options::Options};

pub async fn show(_: &Options) -> Option<Chunk<SmolStr>> {
    let ssh_connection = std::env::var("SSH_CONNECTION").ok()?;
    let parts: Vec<&str> = ssh_connection.split_whitespace().collect();
    if parts.len() >= 4 {
        Some(Chunk::new(
            "⇄",
            format_smolstr!("{}:{}", parts[2], parts[3]),
        ))
    } else {
        None
    }
}
