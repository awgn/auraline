use smol_str::{format_smolstr, SmolStr};

use crate::{chunk::Chunk, options::Options};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatteryStatus {
    Charging,
    Discharging,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatteryInfo {
    percent: u8,
    status: BatteryStatus,
}

fn select_battery_icon(info: BatteryInfo, nerd_font: bool) -> &'static str {
    if nerd_font {
        match info.status {
            BatteryStatus::Charging => "󰂄",
            BatteryStatus::Discharging | BatteryStatus::Full => {
                if info.percent >= 80 {
                    "󰁹"
                } else if info.percent >= 50 {
                    "󰁾"
                } else if info.percent >= 20 {
                    "󰁼"
                } else {
                    "󰂃"
                }
            }
        }
    } else {
        match info.status {
            BatteryStatus::Charging => "⌁",
            BatteryStatus::Full => "▮",
            BatteryStatus::Discharging => {
                if info.percent <= 20 {
                    "▯"
                } else {
                    "▮"
                }
            }
        }
    }
}

#[cfg(any(target_os = "linux", test))]
fn read_trimmed_u64(path: &std::path::Path) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

#[cfg(any(target_os = "linux", test))]
fn is_system_battery_dir(path: &std::path::Path, name: &str) -> bool {
    if let Ok(scope) = std::fs::read_to_string(path.join("scope")) {
        if scope.trim().eq_ignore_ascii_case("Device") {
            return false;
        }
    }

    if name.starts_with("BAT") || name.starts_with("CMB") || name == "macsmc-battery" {
        return true;
    }

    std::fs::read_to_string(path.join("type"))
        .ok()
        .is_some_and(|t| t.trim().eq_ignore_ascii_case("Battery"))
}

#[cfg(any(target_os = "linux", test))]
fn read_linux_power_supply(base: &std::path::Path) -> Option<BatteryInfo> {
    let entries = std::fs::read_dir(base).ok()?;

    let mut total_now: u64 = 0;
    let mut total_full: u64 = 0;
    let mut any_charging = false;
    let mut any_discharging = false;
    let mut found = false;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let path = entry.path();
        if !is_system_battery_dir(&path, name) {
            continue;
        }

        if let Some(present) = read_trimmed_u64(&path.join("present")) {
            if present == 0 {
                continue;
            }
        }

        let pair = read_trimmed_u64(&path.join("energy_now"))
            .zip(read_trimmed_u64(&path.join("energy_full")))
            .filter(|(_, full)| *full > 0)
            .or_else(|| {
                read_trimmed_u64(&path.join("charge_now"))
                    .zip(read_trimmed_u64(&path.join("charge_full")))
                    .filter(|(_, full)| *full > 0)
            })
            .or_else(|| read_trimmed_u64(&path.join("capacity")).map(|cap| (cap.min(100), 100)));

        let Some((now_val, full_val)) = pair else {
            continue;
        };

        found = true;
        total_now = total_now.saturating_add(now_val);
        total_full = total_full.saturating_add(full_val);

        if let Ok(status_raw) = std::fs::read_to_string(path.join("status")) {
            let status = status_raw.trim();
            if status.eq_ignore_ascii_case("Charging") {
                any_charging = true;
            } else if status.eq_ignore_ascii_case("Discharging") {
                any_discharging = true;
            }
        }
    }

    if !found || total_full == 0 {
        return None;
    }

    let pct = ((total_now as f64 / total_full as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    let status = if any_charging {
        BatteryStatus::Charging
    } else if any_discharging {
        BatteryStatus::Discharging
    } else {
        BatteryStatus::Full
    };

    Some(BatteryInfo {
        percent: pct,
        status,
    })
}

#[cfg(any(target_os = "macos", test))]
fn parse_pmset_batt(output: &str) -> Option<BatteryInfo> {
    let on_battery_power = output
        .lines()
        .next()
        .is_some_and(|l| l.contains("Battery Power"));

    for line in output.lines() {
        let Some(pos) = line.find("InternalBattery") else {
            continue;
        };
        let rest = &line[pos..];
        let pct_idx = rest.find('%')?;
        let before_pct = &rest[..pct_idx];
        let digits_start = before_pct
            .rfind(|c: char| !c.is_ascii_digit())
            .map_or(0, |i| i + 1);
        let percent = before_pct[digits_start..].parse::<u8>().ok()?.min(100);

        let after_pct = rest[pct_idx + 1..].trim_start_matches(';').trim();
        let state_seg = after_pct.split(';').next().unwrap_or("").trim();

        let status = if state_seg.eq_ignore_ascii_case("discharging") {
            BatteryStatus::Discharging
        } else if state_seg.eq_ignore_ascii_case("charging")
            || state_seg.eq_ignore_ascii_case("finishing charge")
        {
            BatteryStatus::Charging
        } else if state_seg.eq_ignore_ascii_case("charged")
            || state_seg.eq_ignore_ascii_case("ac attached")
            || state_seg.eq_ignore_ascii_case("not charging")
        {
            BatteryStatus::Full
        } else if on_battery_power {
            BatteryStatus::Discharging
        } else {
            BatteryStatus::Full
        };

        return Some(BatteryInfo { percent, status });
    }

    None
}

#[cfg(target_os = "macos")]
async fn read_macos_battery() -> Option<BatteryInfo> {
    use std::time::{SystemTime, UNIX_EPOCH};

    const CACHE_TTL_SECS: u64 = 60;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // SAFETY: `geteuid` is thread-safe and takes no arguments.
    let uid = unsafe { libc::geteuid() };
    let cache_path = std::env::temp_dir().join(format!("auraline-battery-{uid}.cache"));

    if let Ok(cached) = std::fs::read_to_string(&cache_path) {
        let mut parts = cached.split_ascii_whitespace();
        if let Some(ts) = parts.next().and_then(|s| s.parse::<u64>().ok()) {
            if now_secs.saturating_sub(ts) < CACHE_TTL_SECS {
                match parts.next() {
                    Some("none") => return None,
                    Some(pct_str) => {
                        if let (Ok(pct), Some(st_code)) = (pct_str.parse::<u8>(), parts.next()) {
                            let status = match st_code {
                                "C" => Some(BatteryStatus::Charging),
                                "D" => Some(BatteryStatus::Discharging),
                                "F" => Some(BatteryStatus::Full),
                                _ => None,
                            };
                            if let Some(status) = status {
                                return Some(BatteryInfo {
                                    percent: pct.min(100),
                                    status,
                                });
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }

    let out = crate::cmd::CMD.exec("pmset", ["-g", "batt"]).await?;
    let info = parse_pmset_batt(&out);

    let serialized = match info {
        Some(b) => {
            let code = match b.status {
                BatteryStatus::Charging => 'C',
                BatteryStatus::Discharging => 'D',
                BatteryStatus::Full => 'F',
            };
            format_smolstr!("{now_secs} {} {code}\n", b.percent)
        }
        None => format_smolstr!("{now_secs} none\n"),
    };
    let _ = std::fs::write(&cache_path, serialized.as_bytes());

    info
}

#[cfg(target_os = "linux")]
async fn read_system_battery() -> Option<BatteryInfo> {
    read_linux_power_supply(std::path::Path::new("/sys/class/power_supply"))
}

#[cfg(target_os = "macos")]
async fn read_system_battery() -> Option<BatteryInfo> {
    read_macos_battery().await
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
async fn read_system_battery() -> Option<BatteryInfo> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.battery {
        return None;
    }
    let info = read_system_battery().await?;
    let icon = select_battery_icon(info, opts.nerd_font);
    Some(Chunk::new(icon, format_smolstr!("{}%", info.percent)))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_pmset_batt, read_linux_power_supply, select_battery_icon, BatteryInfo, BatteryStatus,
    };

    #[test]
    fn test_parse_pmset_batt_variants() {
        let charging = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=123)\t82%; charging; 0:45 remaining present: true\n";
        assert_eq!(
            parse_pmset_batt(charging),
            Some(BatteryInfo {
                percent: 82,
                status: BatteryStatus::Charging
            })
        );

        let discharging = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=123)\t47%; discharging; 4:12 remaining present: true\n";
        assert_eq!(
            parse_pmset_batt(discharging),
            Some(BatteryInfo {
                percent: 47,
                status: BatteryStatus::Discharging
            })
        );

        let optimized_hold = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=123)\t80%; AC attached; not charging present: true\n";
        assert_eq!(
            parse_pmset_batt(optimized_hold),
            Some(BatteryInfo {
                percent: 80,
                status: BatteryStatus::Full
            })
        );

        let desktop = "Now drawing from 'AC Power'\n";
        assert_eq!(parse_pmset_batt(desktop), None);
    }

    #[test]
    fn test_linux_multi_battery_and_peripheral_exclusion() {
        let dir = std::env::temp_dir().join(format!("auraline-test-batt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let bat0 = dir.join("BAT0");
        let bat1 = dir.join("BAT1");
        let mouse = dir.join("hid-0011-battery");
        std::fs::create_dir_all(&bat0).unwrap();
        std::fs::create_dir_all(&bat1).unwrap();
        std::fs::create_dir_all(&mouse).unwrap();

        // BAT0: 30 Wh / 50 Wh, Discharging
        std::fs::write(bat0.join("energy_now"), "30000000\n").unwrap();
        std::fs::write(bat0.join("energy_full"), "50000000\n").unwrap();
        std::fs::write(bat0.join("status"), "Discharging\n").unwrap();

        // BAT1: 45 Wh / 50 Wh, Full -> combined 75 / 100 = 75%, Discharging
        std::fs::write(bat1.join("energy_now"), "45000000\n").unwrap();
        std::fs::write(bat1.join("energy_full"), "50000000\n").unwrap();
        std::fs::write(bat1.join("status"), "Full\n").unwrap();

        // Bluetooth mouse battery (`scope = Device`) must be ignored!
        std::fs::write(mouse.join("type"), "Battery\n").unwrap();
        std::fs::write(mouse.join("scope"), "Device\n").unwrap();
        std::fs::write(mouse.join("capacity"), "10\n").unwrap();
        std::fs::write(mouse.join("status"), "Discharging\n").unwrap();

        let info = read_linux_power_supply(&dir).unwrap();
        assert_eq!(
            info,
            BatteryInfo {
                percent: 75,
                status: BatteryStatus::Discharging
            }
        );
        assert_eq!(select_battery_icon(info, false), "▮");
        assert_eq!(select_battery_icon(info, true), "󰁾");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
