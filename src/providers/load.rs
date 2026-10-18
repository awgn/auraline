use crate::{
    chunk::{Adjoin, Chunk},
    options::Options,
};
use smol_str::{format_smolstr, SmolStr};

pub async fn show(opts: &Options) -> Option<Chunk<Adjoin<&'static str, SmolStr>>> {
    if !opts.load {
        return None;
    }
    let avg = sysinfo::System::load_average();
    Some(Chunk::info(Adjoin((
        "λ",
        format_smolstr!("{:.2}", avg.one),
    ))))
}
