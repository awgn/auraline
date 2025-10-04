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
    let cmd_start_time = format!("/tmp/{}.{}", AURALINE_CMD_START, ppid);
    let start_time_str = tokio::fs::read_to_string(&cmd_start_time).await.ok()?;
    defer! {
        let _ = std::fs::remove_file(&cmd_start_time);
    };
    let start_nanos = start_time_str.parse::<u128>().ok()?;
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
