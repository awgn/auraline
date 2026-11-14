use smol_str::{SmolStr, SmolStrBuilder};

use crate::{chunk::Chunk, options::Options};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.ssh {
        return None;
    }

    let ssh_connection = std::env::var("SSH_CONNECTION").ok()?;
    let mut parts = ssh_connection.split_whitespace();
    let (_, _, Some(server_ip), Some(server_port)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };

    let mut builder = SmolStrBuilder::new();
    builder.push_str(server_ip);
    builder.push(':');
    builder.push_str(server_port);

    Some(Chunk::new("⇄", builder.finish()))
}
