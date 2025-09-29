use itertools::Itertools;
use smallvec::SmallVec;
use smol_str::{SmolStr, ToSmolStr};
use std::collections::BTreeSet;

use crate::{chunk::Chunk, commands::Options};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.netif {
        return None;
    }

    let ifaddrs = if_addrs::get_if_addrs().ok()?;
    let uniq: BTreeSet<SmolStr> = ifaddrs
        .iter()
        .filter_map(|ni| {
            matches!(ni.oper_status, if_addrs::IfOperStatus::Up).then_some(ni.name.to_smolstr())
        })
        .collect::<BTreeSet<_>>();
    let names = uniq.iter().collect::<SmallVec<[_; 8]>>();
    Some(Chunk::new(
        "󰛳",
        names.iter().copied().join(",").to_smolstr(),
    )) // FIXME: avoid allocation here
}
