use crate::{chunk::Chunk, options::Options, sys};
use smol_str::{format_smolstr, SmolStr};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.memory {
        return None;
    }
    let mem_perc = sys::memory_usage_percent()?;
    Some(Chunk::new(
        opts.select_str("μ", ""),
        format_smolstr!("{:.1}%", mem_perc),
    ))
}
