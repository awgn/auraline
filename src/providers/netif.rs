use itertools::Itertools;
use smol_str::{SmolStr, ToSmolStr};
use std::collections::BTreeSet;

use crate::options::Options;

pub async fn show(_: &Options) -> Option<SmolStr> {
    let ifaddrs = if_addrs::get_if_addrs().ok()?;
    let uniq: BTreeSet<SmolStr> = ifaddrs
        .iter()
        .filter_map(|ni| {
            matches!(ni.oper_status, if_addrs::IfOperStatus::Up).then_some(ni.name.to_smolstr())
        })
        .collect::<BTreeSet<_>>();
    let names: Vec<&SmolStr> = uniq.iter().collect();
    Some(names.iter().copied().join(",").to_smolstr()) // FIXME: avoid allocation here
}
