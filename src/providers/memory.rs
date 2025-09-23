use crate::options::Options;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};
use tokio::fs;
use std::path::Path;

pub async fn show(_: &Options) -> Option<SmolStr> {
    let info = sysinfo::System::new_all();
    let mem_perc = info.used_memory() as f64 / info.total_memory() as f64 * 100.0;
    let huge_pages = get_hugepages_status().await;
    let mut builder = SmolStrBuilder::new();
    builder.push_str(&format_smolstr!("{:.1}%", mem_perc));
    if let Some(huge_pages) = huge_pages {
        for hp in huge_pages {
            for page in hp.pages {
                builder.push_str(&format_smolstr!(" 󰽿{}x{}", page.count, format_kb(page.size_kb)));
            }
        }
    }
    Some(builder.finish())
}

/// Contains information about a specific size of huge pages for a NUMA node.
#[derive(Debug)]
pub struct HugePageInfo {
    pub size_kb: u64,
    pub count: u64,
}

/// Represents all huge page configurations for a single NUMA node.
/// If `node` is None, it represents a non-NUMA system.
#[derive(Debug)]
#[allow(dead_code)]
pub struct HugePage {
    pub node: Option<u32>,
    pub pages: Vec<HugePageInfo>,
}

/// Processes a specific hugepages directory (either system-wide or per-node)
/// and collects information about allocated pages.
async fn process_hugepage_dir(path: &Path, node: Option<u32>) -> Option<HugePage> {
    // This vector will store details for each page size (e.g., 2MB, 1GB).
    let mut pages_info = Vec::new();

    // Read the directory content, continue only if successful.
    if let Ok(mut entries) = fs::read_dir(path).await {
        let mut tmp = entries.next_entry().await.ok()?.into_iter();
        for entry in &mut tmp {
            let file_name = entry.file_name();
            let file_name_str = match file_name.to_str() {
                Some(s) => s,
                None => continue, // Skip if the filename is not valid UTF-8.
            };

            // Kernel directories are named like "hugepages-2048kB".
            if file_name_str.starts_with("hugepages-") {
                // Extract size in KB from the directory name.
                if let Some(size_str) = file_name_str.strip_prefix("hugepages-").and_then(|s| s.strip_suffix("kB")) {
                    if let Ok(size_kb) = size_str.parse::<u64>() {
                        // Path to the file that contains the number of allocated pages.
                        let nr_hugepages_path = entry.path().join("nr_hugepages");

                        if let Ok(content) = fs::read_to_string(nr_hugepages_path).await {
                            // Parse the number of pages.
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
    }

    if !pages_info.is_empty() {
        Some(HugePage { node, pages: pages_info })
    } else {
        None
    }
}

/// Scans the system for huge page configurations and returns their status.
///
/// It checks for NUMA support and scans either per-node directories or the
/// global hugepages directory.
/// Returns `Some(Vec<HugePage>)` if configurations are found, otherwise `None`.
pub async fn get_hugepages_status() -> Option<Vec<HugePage>> {
    const NUMA_NODE_BASE_DIR: &str = "/sys/devices/system/node";
    const NO_NUMA_HUGE_DIR: &str = "/sys/kernel/mm/hugepages";

    let mut hugepages: Vec<HugePage> = Vec::new();

    // Check if the system supports NUMA by checking for the base directory's existence.
    if Path::new(NUMA_NODE_BASE_DIR).exists() {
        // NUMA system: iterate through node directories (e.g., node0, node1, ...).
        if let Ok(mut entries) = fs::read_dir(NUMA_NODE_BASE_DIR).await {
            while let Some(entry) = entries.next_entry().await.ok()? {
                let file_name = entry.file_name();
                let file_name_str = match file_name.to_str() {
                    Some(s) => s,
                    None => continue,
                };

                // Check if the directory name matches the "nodeX" pattern.
                if let Some(node_id) = file_name_str.strip_prefix("node") {
                    if let Ok(node_id) = node_id.parse::<u32>() {
                        let hugepages_path = entry.path().join("hugepages");
                        if let Some(hp) = process_hugepage_dir(&hugepages_path, Some(node_id)).await {
                            hugepages.push(hp);
                        }
                    }
                }
            }
        }
    } else {
        // Non-NUMA system: check the single global directory.
        let path = Path::new(NO_NUMA_HUGE_DIR);
        if let Some(hp) = process_hugepage_dir(path, None).await {
            hugepages.push(hp);
        }
    }

    if hugepages.is_empty() {
        None
    } else {
        Some(hugepages)
    }
}

/// Helper function to format size in kilobytes into a human-readable string (KB, MB, GB).
fn format_kb(kb: u64) -> String {
    if kb < 1024 {
        format!("{}KB", kb)
    } else if kb < 1024 * 1024 {
        format!("{}MB", kb / 1024)
    } else {
        format!("{}GB", kb / (1024 * 1024))
    }
}
