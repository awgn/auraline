use crate::{chunk::Chunk, options::Options, style::to_subscript};
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};
use std::path::Path;

/// Contains information about a specific size of huge pages for a NUMA node.
#[derive(Debug)]
pub struct HugePageInfo {
    pub size_kb: u64,
    pub count: u64,
}

/// Represents all huge page configurations for a single NUMA node.
/// If `node` is None, it represents a non-NUMA system.
#[derive(Debug)]
pub struct HugePage {
    pub node: Option<u32>,
    pub pages: SmallVec<[HugePageInfo; 4]>,
}

/// Processes a specific hugepages directory (either system-wide or per-node)
/// and collects information about allocated pages.
#[allow(dead_code)]
fn process_hugepage_dir(path: &Path, node: Option<u32>) -> Option<HugePage> {
    let mut pages_info = SmallVec::<[HugePageInfo; 4]>::new();

    if let Ok(entries) = std::fs::read_dir(path) {
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
    }

    if !pages_info.is_empty() {
        pages_info.sort_unstable_by_key(|p| p.size_kb);
        Some(HugePage {
            node,
            pages: pages_info,
        })
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
pub fn get_hugepages_status() -> Option<SmallVec<[HugePage; 4]>> {
    const NUMA_NODE_BASE_DIR: &str = "/sys/devices/system/node";
    const NO_NUMA_HUGE_DIR: &str = "/sys/kernel/mm/hugepages";

    let mut hugepages = SmallVec::<[HugePage; 4]>::new();

    if let Ok(entries) = std::fs::read_dir(NUMA_NODE_BASE_DIR) {
        let mut nodes = SmallVec::<[(u32, std::path::PathBuf); 4]>::new();
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(file_name_str) = file_name.to_str() else {
                continue;
            };
            if let Some(node_id) = file_name_str
                .strip_prefix("node")
                .and_then(|s| s.parse::<u32>().ok())
            {
                nodes.push((node_id, entry.path().join("hugepages")));
            }
        }
        nodes.sort_unstable_by_key(|(id, _)| *id);
        let multi_numa = nodes.len() > 1;
        for (node_id, hugepages_path) in nodes {
            if let Some(hp) = process_hugepage_dir(&hugepages_path, multi_numa.then_some(node_id)) {
                hugepages.push(hp);
            }
        }
    } else if let Some(hp) = process_hugepage_dir(Path::new(NO_NUMA_HUGE_DIR), None) {
        hugepages.push(hp);
    }

    if hugepages.is_empty() {
        None
    } else {
        Some(hugepages)
    }
}

#[cfg(not(target_os = "linux"))]
pub fn get_hugepages_status() -> Option<SmallVec<[HugePage; 4]>> {
    None
}

fn format_hugepages(huge_pages: &[HugePage]) -> SmolStr {
    let mut builder = SmolStrBuilder::new();
    let mut first = true;
    let mut buf = itoa::Buffer::new();

    for hp in huge_pages {
        let prefix = hp.node.map(|n| to_subscript(buf.format(n)));
        for page in &hp.pages {
            if !first {
                builder.push(',');
            }
            first = false;
            if let Some(ref sub) = prefix {
                builder.push_str(sub);
            }
            builder.push_str(&format_smolstr!("{}x{}", page.count, format_kb(page.size_kb)));
        }
    }

    builder.finish()
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.huge_pages {
        return None;
    }
    let huge_pages = get_hugepages_status()?;
    Some(Chunk::new(
            opts.select_str("⎘", "\u{f0d7f}"),
        format_hugepages(&huge_pages),
    ))
}

/// Helper function to format size in kilobytes into a human-readable string (KB, MB, GB).
fn format_kb(kb: u64) -> SmolStr {
    if kb < 1024 {
        format_smolstr!("{}KB", kb)
    } else if kb < 1024 * 1024 {
        format_smolstr!("{}MB", kb / 1024)
    } else {
        format_smolstr!("{}GB", kb / (1024 * 1024))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_hugepages, HugePage, HugePageInfo};
    use smallvec::smallvec;

    #[test]
    fn test_format_hugepages_single_and_multi_numa() {
        let single = [HugePage {
            node: None,
            pages: smallvec![HugePageInfo {
                size_kb: 2048,
                count: 8
            }],
        }];
        assert_eq!(format_hugepages(&single), "8x2MB");

        let multi = [
            HugePage {
                node: Some(0),
                pages: smallvec![HugePageInfo {
                    size_kb: 2048,
                    count: 8
                }],
            },
            HugePage {
                node: Some(1),
                pages: smallvec![HugePageInfo {
                    size_kb: 2048,
                    count: 8
                }],
            },
        ];
        assert_eq!(format_hugepages(&multi), "₀8x2MB,₁8x2MB");
    }
}
