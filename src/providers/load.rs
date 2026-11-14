use crate::{
    chunk::{Adjoin, Chunk},
    options::Options,
    sys,
};
use smol_str::{format_smolstr, SmolStr};

pub async fn show(opts: &Options) -> Option<Chunk<Adjoin<&'static str, SmolStr>>> {
    if !opts.load {
        return None;
    }
    let avg = sys::load_average_one()?;
    Some(Chunk::info(Adjoin(("λ", format_smolstr!("{:.2}", avg)))))
}
