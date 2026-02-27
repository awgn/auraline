use smol_str::{SmolStr, SmolStrBuilder, ToSmolStr};
use std::collections::BTreeSet;

use crate::{chunk::Chunk, options::Options};

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

    let mut builder = SmolStrBuilder::new();
    let mut first = true;
    for name in uniq {
        if !first {
            builder.push(',');
        }
        builder.push_str(&name);
        first = false;
    }

    Some(Chunk::new("󰛳", builder.finish()))
}
