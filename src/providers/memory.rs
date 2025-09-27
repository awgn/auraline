use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr};
use sysinfo::{MemoryRefreshKind, RefreshKind};

pub async fn show(opt: &Options) -> Option<Chunk<SmolStr>> {
    let info = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    let mem_perc = info.used_memory() as f64 / info.total_memory() as f64 * 100.0;
    Some(Chunk::new(opt.select_str("μ", ""), format_smolstr!("{:.1}%", mem_perc)))
}
