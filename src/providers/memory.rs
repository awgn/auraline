use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr};
use sysinfo::{MemoryRefreshKind, RefreshKind};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.memory {
        return None;
    }
    let info = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    let mem_perc = info.used_memory() as f64 / info.total_memory() as f64 * 100.0;
    Some(Chunk::new(
        opts.select_str("μ", ""),
        format_smolstr!("{:.1}%", mem_perc),
    ))
}
