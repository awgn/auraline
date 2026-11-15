use smol_str::SmolStr;

use crate::{chunk::Chunk, options::Options};

#[cfg(any(target_os = "linux", test))]
const PSI_ALERT_THRESHOLD: f64 = 5.0;

#[cfg(any(target_os = "linux", test))]
fn parse_psi_some_avg10(content: &str) -> Option<f64> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("some ") {
            for field in rest.split_ascii_whitespace() {
                if let Some(val) = field.strip_prefix("avg10=") {
                    return val.parse::<f64>().ok();
                }
            }
        }
    }
    None
}

#[cfg(any(target_os = "linux", test))]
fn format_psi_alerts(cpu: Option<f64>, mem: Option<f64>, io: Option<f64>) -> Option<SmolStr> {
    use smol_str::{format_smolstr, SmolStrBuilder};

    let mut builder = SmolStrBuilder::new();
    let mut first = true;

    for (label, val) in [("cpu", cpu), ("mem", mem), ("io", io)] {
        if let Some(pct) = val.filter(|v| *v >= PSI_ALERT_THRESHOLD) {
            if !first {
                builder.push(',');
            }
            first = false;
            builder.push_str(&format_smolstr!("{label}:{:.0}%", pct));
        }
    }

    (!first).then(|| builder.finish())
}

#[cfg(any(target_os = "linux", test))]
fn collect_psi_from_dir(dir: &std::path::Path) -> Option<SmolStr> {
    let read_avg10 = |name: &str| -> Option<f64> {
        let content = std::fs::read_to_string(dir.join(name)).ok()?;
        parse_psi_some_avg10(&content)
    };
    format_psi_alerts(read_avg10("cpu"), read_avg10("memory"), read_avg10("io"))
}

#[cfg(target_os = "linux")]
#[inline]
fn read_system_psi() -> Option<SmolStr> {
    collect_psi_from_dir(std::path::Path::new("/proc/pressure"))
}

#[cfg(not(target_os = "linux"))]
#[inline]
fn read_system_psi() -> Option<SmolStr> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.psi {
        return None;
    }
    let summary = read_system_psi()?;
    Some(Chunk::new(opts.select_str("⟁", "󰀦"), summary))
}

#[cfg(test)]
mod tests {
    use super::{collect_psi_from_dir, format_psi_alerts, parse_psi_some_avg10};

    #[test]
    fn test_parse_and_format_psi() {
        let sample = "some avg10=18.24 avg60=4.12 avg300=1.05 total=918273\n\
                      full avg10=9.10 avg60=1.00 avg300=0.20 total=401922\n";
        assert_eq!(parse_psi_some_avg10(sample), Some(18.24));
        assert_eq!(format_psi_alerts(Some(1.2), Some(0.0), Some(4.99)), None);
        assert_eq!(
            format_psi_alerts(Some(2.0), Some(0.4), Some(18.24)).as_deref(),
            Some("io:18%")
        );
        assert_eq!(
            format_psi_alerts(Some(12.4), Some(7.6), Some(18.24)).as_deref(),
            Some("cpu:12%,mem:8%,io:18%")
        );
    }

    #[test]
    fn test_collect_psi_from_dir() {
        let dir = std::env::temp_dir().join(format!("auraline-test-psi-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("cpu"),
            "some avg10=1.10 avg60=0.50 avg300=0.10 total=100\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("memory"),
            "some avg10=0.00 avg60=0.00 avg300=0.00 total=0\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("io"),
            "some avg10=21.60 avg60=5.00 avg300=1.00 total=5000\nfull avg10=14.20 avg60=3.00 avg300=0.50 total=3000\n",
        )
        .unwrap();

        assert_eq!(collect_psi_from_dir(&dir).as_deref(), Some("io:22%"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
