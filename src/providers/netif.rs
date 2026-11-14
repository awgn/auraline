use smol_str::{SmolStr, SmolStrBuilder};
use std::collections::BTreeSet;

use crate::{chunk::Chunk, options::Options};

#[cfg(unix)]
fn active_interfaces() -> Option<BTreeSet<SmolStr>> {
    use scopeguard::defer;
    use std::ffi::CStr;
    use std::net::Ipv6Addr;

    let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: `getifaddrs` populates `addrs` on return 0.
    if unsafe { libc::getifaddrs(&mut addrs) } != 0 || addrs.is_null() {
        return None;
    }
    defer! {
        // SAFETY: `addrs` was allocated by `getifaddrs`.
        unsafe { libc::freeifaddrs(addrs) };
    }

    let up_mask = libc::IFF_UP as u32;
    let running_mask = libc::IFF_RUNNING as u32;
    let loopback_mask = libc::IFF_LOOPBACK as u32;

    let mut uniq = BTreeSet::new();
    let mut cur = addrs;
    while !cur.is_null() {
        // SAFETY: `cur` points to a valid `ifaddrs` node in the linked list.
        let ifa = unsafe { &*cur };
        cur = ifa.ifa_next;

        let flags = ifa.ifa_flags;
        if (flags & up_mask) == 0 || (flags & running_mask) == 0 || (flags & loopback_mask) != 0 {
            continue;
        }

        if ifa.ifa_addr.is_null() || ifa.ifa_name.is_null() {
            continue;
        }

        // SAFETY: `ifa_addr` is non-null.
        let family = unsafe { (*ifa.ifa_addr).sa_family } as libc::c_int;
        let has_routable_addr = match family {
            libc::AF_INET => {
                // SAFETY: `sa_family == AF_INET` guarantees `sockaddr_in` size;
                // `read_unaligned` avoids assuming stricter alignment than `sockaddr`.
                let sin =
                    unsafe { std::ptr::read_unaligned(ifa.ifa_addr.cast::<libc::sockaddr_in>()) };
                let ip4 = std::net::Ipv4Addr::from(u32::from_be(sin.sin_addr.s_addr));
                !ip4.is_loopback()
            }
            libc::AF_INET6 => {
                // SAFETY: `sa_family == AF_INET6` guarantees `sockaddr_in6` size;
                // `read_unaligned` avoids assuming stricter alignment than `sockaddr`.
                let sin6 =
                    unsafe { std::ptr::read_unaligned(ifa.ifa_addr.cast::<libc::sockaddr_in6>()) };
                let ip6 = Ipv6Addr::from(sin6.sin6_addr.s6_addr);
                !ip6.is_unicast_link_local() && !ip6.is_loopback()
            }
            _ => false,
        };

        if !has_routable_addr {
            continue;
        }

        // SAFETY: `ifa_name` is a valid NUL-terminated string provided by `getifaddrs`.
        if let Ok(name) = unsafe { CStr::from_ptr(ifa.ifa_name) }.to_str() {
            if !name.is_empty() {
                uniq.insert(SmolStr::new(name));
            }
        }
    }

    Some(uniq)
}

#[cfg(not(unix))]
fn active_interfaces() -> Option<BTreeSet<SmolStr>> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.netif {
        return None;
    }

    let uniq = active_interfaces()?;
    if uniq.is_empty() {
        return None;
    }

    let mut builder = SmolStrBuilder::new();
    let mut first = true;
    for name in uniq {
        if !first {
            builder.push(',');
        }
        builder.push_str(&name);
        first = false;
    }

    Some(Chunk::new(opts.select_str("☍", "󰛳"), builder.finish()))
}
