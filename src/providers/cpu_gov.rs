use smol_str::SmolStr;

use crate::{chunk::Chunk, options::Options};

#[cfg(any(target_os = "linux", test))]
fn parse_governor(raw: &str) -> Option<SmolStr> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| SmolStr::new(trimmed))
}

#[cfg(any(target_os = "linux", test))]
fn read_governor_from_paths(paths: &[&std::path::Path]) -> Option<SmolStr> {
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(gov) = parse_governor(&content) {
                return Some(gov);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
#[inline]
fn read_system_governor() -> Option<SmolStr> {
    use std::path::Path;
    read_governor_from_paths(&[
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"),
        Path::new("/sys/devices/system/cpu/cpufreq/policy0/scaling_governor"),
    ])
}

#[cfg(not(target_os = "linux"))]
#[inline]
fn read_system_governor() -> Option<SmolStr> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.cpu_gov {
        return None;
    }
    let gov = read_system_governor()?;
    Some(Chunk::new(opts.select_str("⦿", "󰓅"), gov))
}

#[cfg(test)]
mod tests {
    use super::{parse_governor, read_governor_from_paths};

    #[test]
    fn test_parse_and_read_governor() {
        assert_eq!(parse_governor("  \n"), None);
        assert_eq!(
            parse_governor("performance\n").as_deref(),
            Some("performance")
        );

        let dir = std::env::temp_dir().join(format!("auraline-test-cpugov-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let missing = dir.join("missing");
        let fallback = dir.join("scaling_governor");
        std::fs::write(&fallback, "schedutil\n").unwrap();

        assert_eq!(
            read_governor_from_paths(&[missing.as_path(), fallback.as_path()]).as_deref(),
            Some("schedutil")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
