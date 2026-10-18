use crate::{chunk::Chunk, options::Options};
use smol_str::{format_smolstr, SmolStr};

#[cfg(unix)]
fn disk_usage_percent() -> Option<f64> {
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c".".as_ptr(), &mut stat) != 0 {
            return None;
        }
        let total = stat.f_blocks as u64;
        if total == 0 {
            return None;
        }
        let avail = stat.f_bavail as u64;
        let used = total.saturating_sub(avail);
        Some(used as f64 / total as f64 * 100.0)
    }
}

#[cfg(not(unix))]
fn disk_usage_percent() -> Option<f64> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.disk {
        return None;
    }
    let perc = disk_usage_percent()?;
    Some(Chunk::new(
        opts.select_str("⛁", "󰋊"),
        format_smolstr!("{:.1}%", perc),
    ))
}
