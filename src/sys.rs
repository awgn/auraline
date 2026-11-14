use crate::cmd::CMD;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};
use std::sync::OnceLock;

/// Cached `/etc/os-release` (or `/usr/lib/os-release`) content shared between
/// `os` and `basic::distro` providers so it is read at most once per run.
pub fn os_release_content() -> Option<&'static str> {
    static OS_RELEASE: OnceLock<Option<String>> = OnceLock::new();
    OS_RELEASE
        .get_or_init(|| {
            std::fs::read_to_string("/etc/os-release")
                .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
                .ok()
        })
        .as_deref()
}

/// 1-minute system load average.
#[cfg(all(unix, not(target_os = "android")))]
pub fn load_average_one() -> Option<f64> {
    let mut avg: [libc::c_double; 1] = [0.0];
    // SAFETY: `avg` points to writable storage for 1 `c_double`.
    let n = unsafe { libc::getloadavg(avg.as_mut_ptr(), 1) };
    (n == 1 && avg[0].is_finite() && avg[0] >= 0.0).then_some(avg[0])
}

#[cfg(target_os = "android")]
pub fn load_average_one() -> Option<f64> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    content.split_whitespace().next()?.parse::<f64>().ok()
}

#[cfg(not(unix))]
pub fn load_average_one() -> Option<f64> {
    None
}

/// Memory usage percentage `[0.0, 100.0]`.
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn memory_usage_percent() -> Option<f64> {
    use std::mem::{size_of, zeroed};
    use std::ptr::null_mut;

    let mut total: u64 = 0;
    let mut total_len = size_of::<u64>();
    // SAFETY: `total` is an aligned `u64` matching `hw.memsize`.
    let ret = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&raw mut total).cast(),
            &mut total_len,
            null_mut(),
            0,
        )
    };
    if ret != 0 || total == 0 {
        return None;
    }

    let mut page_size: u64 = 0;
    let mut ps_len = size_of::<u64>();
    // SAFETY: `hw.pagesize` writes up to 8 bytes into zero-initialized `page_size`.
    let ps_ret = unsafe {
        libc::sysctlbyname(
            c"hw.pagesize".as_ptr(),
            (&raw mut page_size).cast(),
            &mut ps_len,
            null_mut(),
            0,
        )
    };
    if ps_ret != 0 || page_size == 0 {
        // SAFETY: reading global `vm_page_size` initialized by dyld/libSystem.
        page_size = unsafe { libc::vm_page_size } as u64;
        if page_size == 0 {
            return None;
        }
    }

    static HOST_PORT: OnceLock<libc::host_t> = OnceLock::new();
    // SAFETY: `mach_host_self` returns the task's Mach host port; cached once per process.
    let host = *HOST_PORT.get_or_init(|| unsafe { libc::mach_host_self() });

    // SAFETY: `vm_statistics64` is plain integer fields; zero-initialization is valid.
    let mut vm_stat: libc::vm_statistics64 = unsafe { zeroed() };
    let mut count = libc::HOST_VM_INFO64_COUNT;
    // SAFETY: `vm_stat` buffer matches `HOST_VM_INFO64_COUNT` 32-bit integers.
    let kr = unsafe {
        libc::host_statistics64(
            host,
            libc::HOST_VM_INFO64,
            (&raw mut vm_stat).cast(),
            &mut count,
        )
    };
    if kr != libc::KERN_SUCCESS {
        return None;
    }

    // Matches XNU / Activity Monitor / sysinfo: app memory (internal - purgeable) + wired + compressed.
    let app_pages =
        u64::from(vm_stat.internal_page_count).saturating_sub(u64::from(vm_stat.purgeable_count));
    let used_pages = app_pages
        .saturating_add(u64::from(vm_stat.wire_count))
        .saturating_add(u64::from(vm_stat.compressor_page_count));
    let used_bytes = used_pages.saturating_mul(page_size);

    Some((used_bytes as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
}

#[cfg(target_os = "linux")]
pub fn memory_usage_percent() -> Option<f64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let (host_total, host_used) = parse_proc_meminfo(&meminfo)?;

    // Respect cgroup v2 container memory limits when active and lower than host RAM.
    if let (Ok(max_s), Ok(cur_s)) = (
        std::fs::read_to_string("/sys/fs/cgroup/memory.max"),
        std::fs::read_to_string("/sys/fs/cgroup/memory.current"),
    ) {
        let stat_s = std::fs::read_to_string("/sys/fs/cgroup/memory.stat").ok();
        if let Some((cg_total, cg_used)) = parse_cgroup_v2_memory(&max_s, &cur_s, stat_s.as_deref())
        {
            if cg_total < host_total {
                return Some((cg_used as f64 / cg_total as f64 * 100.0).clamp(0.0, 100.0));
            }
        }
    }

    Some((host_used as f64 / host_total as f64 * 100.0).clamp(0.0, 100.0))
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn memory_usage_percent() -> Option<f64> {
    let page_size = sysctl_u64(c"vm.stats.vm.v_page_size")?;
    if page_size == 0 {
        return None;
    }
    let total = sysctl_u64(c"vm.stats.vm.v_page_count")
        .filter(|&p| p > 0)
        .map(|p| p.saturating_mul(page_size))
        .or_else(|| sysctl_u64(c"hw.physmem"))?;
    if total == 0 {
        return None;
    }
    let active = sysctl_u64(c"vm.stats.vm.v_active_count").unwrap_or(0);
    let wire = sysctl_u64(c"vm.stats.vm.v_wire_count").unwrap_or(0);
    let zfs_arc = sysctl_u64(c"kstat.zfs.misc.arcstats.size").unwrap_or(0);
    let wire_bytes = wire.saturating_mul(page_size).saturating_sub(zfs_arc);
    let used = active.saturating_mul(page_size).saturating_add(wire_bytes);
    Some((used as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly"
)))]
pub fn memory_usage_percent() -> Option<f64> {
    None
}

#[cfg(any(target_os = "linux", test))]
fn parse_proc_meminfo(content: &str) -> Option<(u64, u64)> {
    let mut total_kb: Option<u64> = None;
    let mut avail_kb: Option<u64> = None;
    let mut free_kb: u64 = 0;
    let mut buffers_kb: u64 = 0;
    let mut cached_kb: u64 = 0;
    let mut sreclaimable_kb: u64 = 0;
    let mut shmem_kb: u64 = 0;

    for line in content.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let Some(val_str) = rest.split_whitespace().next() else {
            continue;
        };
        let Ok(val) = val_str.parse::<u64>() else {
            continue;
        };
        match key {
            "MemTotal" => total_kb = Some(val),
            "MemAvailable" => avail_kb = Some(val),
            "MemFree" => free_kb = val,
            "Buffers" => buffers_kb = val,
            "Cached" => cached_kb = val,
            "SReclaimable" => sreclaimable_kb = val,
            "Shmem" => shmem_kb = val,
            _ => {}
        }
        if total_kb.is_some() && avail_kb.is_some() {
            break;
        }
    }

    let total = total_kb?.checked_mul(1024)?;
    if total == 0 {
        return None;
    }

    let avail = avail_kb
        .unwrap_or_else(|| {
            free_kb
                .saturating_add(buffers_kb)
                .saturating_add(cached_kb)
                .saturating_add(sreclaimable_kb)
                .saturating_sub(shmem_kb)
        })
        .saturating_mul(1024);

    Some((total, total.saturating_sub(avail)))
}

#[cfg(any(target_os = "linux", test))]
fn parse_cgroup_v2_memory(
    max_str: &str,
    current_str: &str,
    stat_str: Option<&str>,
) -> Option<(u64, u64)> {
    let max = max_str.trim().parse::<u64>().ok()?;
    if max == 0 {
        return None;
    }
    let current = current_str.trim().parse::<u64>().ok()?;
    let inactive_file = stat_str
        .and_then(|s| {
            s.lines().find_map(|line| {
                let rest = line.strip_prefix("inactive_file ")?;
                rest.trim().parse::<u64>().ok()
            })
        })
        .unwrap_or(0);

    Some((max, current.saturating_sub(inactive_file).min(max)))
}

/// System hostname without Objective-C / SystemConfiguration frameworks.
#[cfg(unix)]
pub fn hostname() -> Option<SmolStr> {
    let mut buf = [0u8; 256];
    // SAFETY: `buf` is valid writable memory of 256 bytes.
    let ret = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if ret != 0 {
        return None;
    }
    buf[buf.len() - 1] = 0;
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let s = std::str::from_utf8(&buf[..len]).ok()?.trim();
    (!s.is_empty()).then(|| SmolStr::new(s))
}

#[cfg(not(unix))]
pub fn hostname() -> Option<SmolStr> {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|s| !s.is_empty())
        .map(SmolStr::from)
}

/// Current effective username (`$USER` -> `$LOGNAME` -> `getpwuid_r`).
pub fn username() -> Option<SmolStr> {
    for key in ["USER", "LOGNAME"] {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                return Some(SmolStr::from(val));
            }
        }
    }
    passwd_user_and_gecos().map(|(u, _)| u)
}

/// User's full/real name from passwd GECOS field (`None` if empty).
pub fn realname() -> Option<SmolStr> {
    let (user, gecos) = passwd_user_and_gecos()?;
    parse_gecos(&gecos, &user)
}

fn parse_gecos(gecos: &str, username: &str) -> Option<SmolStr> {
    let full = gecos.split(',').next()?.trim();
    if full.is_empty() {
        return None;
    }
    if !full.contains('&') {
        return Some(SmolStr::new(full));
    }

    let mut builder = SmolStrBuilder::new();
    for ch in full.chars() {
        if ch == '&' {
            let mut u_chars = username.chars();
            if let Some(first) = u_chars.next() {
                for upper in first.to_uppercase() {
                    builder.push(upper);
                }
                builder.push_str(u_chars.as_str());
            }
        } else {
            builder.push(ch);
        }
    }
    let out = builder.finish();
    (!out.is_empty()).then_some(out)
}

#[cfg(unix)]
fn passwd_user_and_gecos() -> Option<(SmolStr, SmolStr)> {
    use std::ffi::CStr;
    use std::mem::zeroed;
    use std::ptr::null_mut;

    // SAFETY: `geteuid` has no preconditions and always succeeds.
    let uid = unsafe { libc::geteuid() };

    let extract = |pwd: &libc::passwd| -> Option<(SmolStr, SmolStr)> {
        if pwd.pw_name.is_null() {
            return None;
        }
        // SAFETY: `pw_name` is a valid C string pointing into the caller buffer.
        let name = unsafe { CStr::from_ptr(pwd.pw_name) }.to_str().ok()?;
        let gecos = if pwd.pw_gecos.is_null() {
            ""
        } else {
            // SAFETY: `pw_gecos` is either null or a valid C string in the buffer.
            unsafe { CStr::from_ptr(pwd.pw_gecos) }
                .to_str()
                .unwrap_or("")
        };
        Some((SmolStr::new(name), SmolStr::new(gecos)))
    };

    let mut stack_buf = [0u8; 1024];
    // SAFETY: `passwd` is plain data initialized by `getpwuid_r`.
    let mut pwd: libc::passwd = unsafe { zeroed() };
    let mut res: *mut libc::passwd = null_mut();
    // SAFETY: `stack_buf` is valid writable memory of `stack_buf.len()` bytes.
    let ret = unsafe {
        libc::getpwuid_r(
            uid,
            &mut pwd,
            stack_buf.as_mut_ptr().cast(),
            stack_buf.len(),
            &mut res,
        )
    };
    if ret == 0 && !res.is_null() {
        return extract(&pwd);
    }
    if ret != libc::ERANGE {
        return None;
    }

    let mut heap_buf = vec![0u8; 16 * 1024];
    res = null_mut();
    // SAFETY: `heap_buf` is valid writable memory of `heap_buf.len()` bytes.
    let ret2 = unsafe {
        libc::getpwuid_r(
            uid,
            &mut pwd,
            heap_buf.as_mut_ptr().cast(),
            heap_buf.len(),
            &mut res,
        )
    };
    if ret2 == 0 && !res.is_null() {
        extract(&pwd)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn passwd_user_and_gecos() -> Option<(SmolStr, SmolStr)> {
    let u = std::env::var("USERNAME").ok().filter(|s| !s.is_empty())?;
    Some((SmolStr::from(u), SmolStr::default()))
}

/// Operating system distribution string (e.g. `"macOS 27.2.1"`, `"Arch Linux"`).
pub fn distro() -> Option<SmolStr> {
    #[cfg(target_os = "macos")]
    {
        for plist in [
            "/System/Library/CoreServices/ServerVersion.plist",
            "/System/Library/CoreServices/SystemVersion.plist",
        ] {
            if let Ok(data) = std::fs::read_to_string(plist) {
                if let Some(d) = parse_macos_version_plist(&data) {
                    return Some(d);
                }
            }
        }
        sysctl_string(c"kern.osproductversion").map(|ver| format_smolstr!("macOS {ver}"))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        os_release_content().and_then(parse_os_release_pretty_name)
    }

    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(any(target_os = "macos", test))]
fn parse_macos_version_plist(data: &str) -> Option<SmolStr> {
    let start = data.find("<dict>")?;
    let end = data.find("</dict>")?;
    if start + "<dict>".len() > end {
        return None;
    }

    let mut product_name: Option<&str> = None;
    let mut user_visible_version: Option<&str> = None;
    let mut pending_name = false;
    let mut pending_version = false;

    for line in data[start + "<dict>".len()..end].lines() {
        let line = line.trim();
        if let Some(key) = line.strip_prefix("<key>") {
            match key.trim_end_matches("</key>") {
                "ProductName" => pending_name = true,
                "ProductUserVisibleVersion" => pending_version = true,
                "ProductVersion" if user_visible_version.is_none() => pending_version = true,
                _ => {}
            }
        } else if let Some(value) = line.strip_prefix("<string>") {
            let v = value.trim_end_matches("</string>");
            if pending_name {
                product_name = Some(v);
                pending_name = false;
            } else if pending_version {
                user_visible_version = Some(v);
                pending_version = false;
            }
        }
    }

    match (product_name, user_visible_version) {
        (Some(name), Some(ver)) => Some(format_smolstr!("{name} {ver}")),
        (Some(name), None) => Some(SmolStr::new(name)),
        (None, Some(ver)) => Some(format_smolstr!("Mac OS (Unknown) {ver}")),
        (None, None) => None,
    }
}

#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn parse_os_release_pretty_name(content: &str) -> Option<SmolStr> {
    let mut pretty = None;
    let mut name = None;
    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
            let v = val.trim_matches(['"', '\'']).trim();
            if !v.is_empty() {
                pretty = Some(v);
                break;
            }
        } else if let Some(val) = line.strip_prefix("NAME=") {
            let v = val.trim_matches(['"', '\'']).trim();
            if !v.is_empty() {
                name = Some(v);
            }
        }
    }
    pretty.or(name).map(SmolStr::new)
}

/// Human-friendly device / computer name (`-d`), lazily queried only when enabled.
/// Returns `None` when no distinct pretty device name is configured so `-d` does not
/// duplicate `@hostname` (`-h`).
pub async fn device_name() -> Option<SmolStr> {
    #[cfg(target_os = "macos")]
    {
        CMD.exec("scutil", ["--get", "ComputerName"])
            .await
            .filter(|name| !name.is_empty())
    }

    #[cfg(target_os = "illumos")]
    {
        let raw = std::fs::read_to_string("/etc/nodename").ok()?;
        let first = raw.lines().next()?;
        (!first.is_empty()).then(|| SmolStr::new(first))
    }

    #[cfg(all(unix, not(any(target_os = "macos", target_os = "illumos"))))]
    {
        let info = std::fs::read_to_string("/etc/machine-info").ok()?;
        parse_machine_info_pretty_hostname(&info)
    }

    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(any(all(unix, not(any(target_os = "macos", target_os = "illumos"))), test))]
fn parse_machine_info_pretty_hostname(content: &str) -> Option<SmolStr> {
    for line in content.lines() {
        let Some((key, raw_val)) = line.split_once('=') else {
            continue;
        };
        if key != "PRETTY_HOSTNAME" {
            continue;
        }
        let unquoted = raw_val
            .strip_prefix('"')
            .unwrap_or(raw_val)
            .strip_suffix('"')
            .unwrap_or(raw_val);
        let mut builder = SmolStrBuilder::new();
        let mut chars = unquoted.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next()? {
                    '\\' => builder.push('\\'),
                    't' => builder.push('\t'),
                    'r' => builder.push('\r'),
                    'n' => builder.push('\n'),
                    '\'' => builder.push('\''),
                    '"' => builder.push('"'),
                    _ => return None,
                }
            } else {
                builder.push(c);
            }
        }
        let out = builder.finish();
        return (!out.is_empty()).then_some(out);
    }
    None
}

/// Returns `Some(true)` if running inside a macOS hypervisor VM, `Some(false)` on bare metal.
#[cfg(target_os = "macos")]
pub fn macos_hv_vmm_present() -> Option<bool> {
    sysctl_u64(c"kern.hv_vmm_present").map(|v| v != 0)
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly"))]
fn sysctl_u64(name: &std::ffi::CStr) -> Option<u64> {
    use std::ptr::null_mut;

    let mut raw = [0u8; 8];
    let mut len = raw.len();
    // SAFETY: `raw` is 8 writable bytes; kernel updates `len` to actual size (4 or 8).
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            raw.as_mut_ptr().cast(),
            &mut len,
            null_mut(),
            0,
        )
    };
    if ret != 0 {
        return None;
    }
    match len {
        4 => Some(u64::from(u32::from_ne_bytes([
            raw[0], raw[1], raw[2], raw[3],
        ]))),
        8 => Some(u64::from_ne_bytes(raw)),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn sysctl_string(name: &std::ffi::CStr) -> Option<SmolStr> {
    use std::ptr::null_mut;

    let mut buf = [0u8; 128];
    let mut len = buf.len();
    // SAFETY: `buf` is 128 writable bytes.
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            null_mut(),
            0,
        )
    };
    if ret != 0 || len == 0 {
        return None;
    }
    let slice = &buf[..len.min(buf.len())];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    let s = std::str::from_utf8(&slice[..end]).ok()?.trim();
    (!s.is_empty()).then(|| SmolStr::new(s))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_cgroup_v2_memory, parse_gecos, parse_machine_info_pretty_hostname,
        parse_macos_version_plist, parse_os_release_pretty_name, parse_proc_meminfo,
    };

    #[test]
    fn test_parse_proc_meminfo_available_and_fallback() {
        let modern = "\
MemTotal:       16384000 kB
MemFree:         2048000 kB
MemAvailable:    8192000 kB
Buffers:          512000 kB
Cached:          4096000 kB
";
        let (total, used) = parse_proc_meminfo(modern).unwrap();
        assert_eq!(total, 16_384_000 * 1024);
        assert_eq!(used, 8_192_000 * 1024);

        let legacy = "\
MemTotal:       8000000 kB
MemFree:        1000000 kB
Buffers:         500000 kB
Cached:         2000000 kB
SReclaimable:    300000 kB
Shmem:           200000 kB
";
        let (total_l, used_l) = parse_proc_meminfo(legacy).unwrap();
        assert_eq!(total_l, 8_000_000 * 1024);
        // avail = 1000000 + 500000 + 2000000 + 300000 - 200000 = 3600000 -> used = 4400000
        assert_eq!(used_l, 4_400_000 * 1024);
    }

    #[test]
    fn test_parse_cgroup_v2_memory() {
        assert_eq!(parse_cgroup_v2_memory("max\n", "1048576\n", None), None);

        let stat = "anon 524288\ninactive_file 262144\nactive_file 131072\n";
        let (max, used) = parse_cgroup_v2_memory("4294967296\n", "1048576\n", Some(stat)).unwrap();
        assert_eq!(max, 4_294_967_296);
        assert_eq!(used, 1_048_576 - 262_144);
    }

    #[test]
    fn test_parse_gecos_os_release_and_plist() {
        assert_eq!(
            parse_gecos("Nicola Bonelli,Room 1,+39", "nicola").as_deref(),
            Some("Nicola Bonelli")
        );
        assert_eq!(
            parse_gecos("& Rossi", "mario").as_deref(),
            Some("Mario Rossi")
        );
        assert_eq!(parse_gecos(",,,", "nicola"), None);

        let os_rel = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\n";
        assert_eq!(
            parse_os_release_pretty_name(os_rel).as_deref(),
            Some("Arch Linux")
        );
        assert_eq!(
            parse_machine_info_pretty_hostname("PRETTY_HOSTNAME=\"Nicola\\'s Workstation\"\n")
                .as_deref(),
            Some("Nicola's Workstation")
        );

        let plist = "<plist><dict>\n<key>ProductName</key>\n<string>macOS</string>\n<key>ProductUserVisibleVersion</key>\n<string>27.2.1</string>\n</dict></plist>";
        assert_eq!(
            parse_macos_version_plist(plist).as_deref(),
            Some("macOS 27.2.1")
        );
    }
}
