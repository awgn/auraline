use smol_str::SmolStr;

use crate::{chunk::Chunk, options::Options};

#[cfg(target_os = "linux")]
fn identify_netns() -> Option<SmolStr> {
    use std::os::unix::fs::MetadataExt;

    let self_meta = std::fs::metadata("/proc/self/ns/net").ok()?;
    let self_dev = self_meta.dev();
    let self_ino = self_meta.ino();

    let entries = std::fs::read_dir("/run/netns")
        .or_else(|_| std::fs::read_dir("/var/run/netns"))
        .ok()?;

    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.dev() == self_dev && meta.ino() == self_ino {
                if let Some(name) = entry.file_name().to_str() {
                    return Some(SmolStr::new(name));
                }
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn identify_netns() -> Option<SmolStr> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.netns {
        return None;
    }
    identify_netns().map(|s| Chunk::new(opts.select_str("{}", ""), s))
}
