use crate::{chunk::Chunk, options::Options};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};

/// Contains information about a specific size of huge pages.
#[derive(Debug)]
pub struct HugePageInfo {
    pub size_kb: u64,
    pub count: u64,
}

#[cfg(target_os = "linux")]
pub fn get_hugepages_status() -> Option<SmallVec<[HugePageInfo; 4]>> {
    const HUGE_DIR: &str = "/sys/kernel/mm/hugepages";

    let mut pages_info = SmallVec::<[HugePageInfo; 4]>::new();
    let entries = std::fs::read_dir(HUGE_DIR).ok()?;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name_str) = file_name.to_str() else {
            continue;
        };

        // Kernel directories are named like "hugepages-2048kB".
        if let Some(size_str) = file_name_str
            .strip_prefix("hugepages-")
            .and_then(|s| s.strip_suffix("kB"))
        {
            if let Ok(size_kb) = size_str.parse::<u64>() {
                let nr_hugepages_path = entry.path().join("nr_hugepages");
                if let Ok(content) = std::fs::read_to_string(nr_hugepages_path) {
                    if let Ok(count) = content.trim().parse::<u64>() {
                        if count > 0 {
                            pages_info.push(HugePageInfo { size_kb, count });
                        }
                    }
                }
            }
        }
    }

    if pages_info.is_empty() {
        None
    } else {
        pages_info.sort_unstable_by_key(|p| p.size_kb);
        Some(pages_info)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn get_hugepages_status() -> Option<SmallVec<[HugePageInfo; 4]>> {
    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.huge_pages {
        return None;
    }
    let huge_pages = get_hugepages_status()?;
    let mut builder = SmolStrBuilder::new();
    for (i, page) in huge_pages.iter().enumerate() {
        if i > 0 {
            builder.push(',');
        }
        builder.push_str(&format_smolstr!("{}x{}", page.count, format_kb(page.size_kb)));
    }

    Some(Chunk::new(
            opts.select_str("⎘", "\u{f0d7f}"),
        builder.finish(),
    ))
}

/// Helper function to format size in kilobytes into a human-readable string (KB, MB, GB).
#[allow(dead_code)]
fn format_kb(kb: u64) -> SmolStr {
    if kb < 1024 {
        format_smolstr!("{}KB", kb)
    } else if kb < 1024 * 1024 {
        format_smolstr!("{}MB", kb / 1024)
    } else {
        format_smolstr!("{}GB", kb / (1024 * 1024))
    }
}
