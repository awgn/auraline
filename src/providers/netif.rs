use std::collections::BTreeSet;
use itertools::Itertools;

use crate::options::Options;

pub async fn show(_: &Options) -> Option<String> {
    let ifaddrs = if_addrs::get_if_addrs().ok()?;
    let uniq : BTreeSet<String> = ifaddrs.iter().filter_map(|ni|
        matches!(ni.oper_status, if_addrs::IfOperStatus::Up).then_some(ni.name.to_owned())
    ).collect::<BTreeSet<_>>();
    let names: Vec<&String> = uniq.iter().collect();
    Some(names.iter().copied().join(","))
}
