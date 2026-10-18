use crate::{chunk::Chunk, options::Options};
use scopeguard::defer;
use smol_str::{format_smolstr, SmolStr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const AURALINE_CMD_START: &str = "auraline_cmd_start";

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.duration {
        return None;
    }
    let end_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let ppid = unsafe { libc::getppid() };
    let (cmd_start_time, start_time_str) =
        read_start_file(ppid).or_else(|| parent_pid(ppid).and_then(read_start_file))?;
    defer! {
        let _ = std::fs::remove_file(&cmd_start_time);
    };
    let start_nanos = start_time_str.trim().parse::<u128>().ok()?;
    if end_nanos > start_nanos {
        let duration = Duration::from_nanos((end_nanos - start_nanos) as u64);
        return Some(format_duration(duration));
    }

    None
}

fn format_duration(duration: Duration) -> Chunk<SmolStr> {
    let secs = duration.as_secs_f64();
    let icon = "󰄉";

    if secs < 0.000001 {
        return Chunk::new(icon, format_smolstr!("{}ns", duration.as_nanos()));
    }
    if secs < 0.001 {
        return Chunk::new(icon, format_smolstr!("{:.0}μs", secs * 1_000_000.0));
    }
    if secs < 1.0 {
        return Chunk::new(icon, format_smolstr!("{:.0}ms", secs * 1_000.0));
    }
    Chunk::new(icon, format_smolstr!("{:.2}s", secs))
}

fn read_start_file(pid: libc::pid_t) -> Option<(String, String)> {
    let path = format!("/tmp/{}.{}", AURALINE_CMD_START, pid);
    let content = std::fs::read_to_string(&path).ok()?;
    Some((path, content))
}

#[cfg(target_os = "macos")]
fn parent_pid(pid: libc::pid_t) -> Option<libc::pid_t> {
    unsafe {
        let mut info: libc::proc_bsdinfo = std::mem::zeroed();
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        let ret = libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            &mut info as *mut _ as *mut libc::c_void,
            size,
        );
        if ret == size && info.pbi_ppid > 1 {
            Some(info.pbi_ppid as libc::pid_t)
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
fn parent_pid(pid: libc::pid_t) -> Option<libc::pid_t> {
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    let after_comm = stat.rfind(')')?.checked_add(2)?;
    let ppid_str = stat.get(after_comm..)?.split_whitespace().nth(1)?;
    let ppid = ppid_str.parse::<libc::pid_t>().ok()?;
    (ppid > 1).then_some(ppid)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn parent_pid(_pid: libc::pid_t) -> Option<libc::pid_t> {
    None
}
