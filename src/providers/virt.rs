use crate::{chunk::Chunk, cmd::CMD, options::Options};
use phf::phf_map;
use smol_str::{SmolStr, ToSmolStr};
use std::path::Path;

#[allow(dead_code)]
pub struct VirtualizationInfo {
    pub icon: &'static str,
    pub color: &'static str,
    pub cterm_color: &'static str, // Terminal 256-color code
    pub name: &'static str,
}

static VIRTUALIZATION_MAP: phf::Map<&'static str, VirtualizationInfo> = phf_map! {
    // ===================================================================
    // Full Virtualization / Hypervisors
    // ===================================================================

    "none" => VirtualizationInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Bare Metal" },
    "kvm" => VirtualizationInfo { icon: "󱗃", color: "#FF0000", cterm_color: "196", name: "KVM" },
    "amazon" => VirtualizationInfo { icon: "", color: "#FF9900", cterm_color: "214", name: "Amazon EC2" },
    "qemu" => VirtualizationInfo { icon: "Ⓠ", color: "#660000", cterm_color: "52", name: "QEMU" },
    "bochs" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Bochs" },
    "xen" => VirtualizationInfo { icon: "Ⓧ", color: "#0083A3", cterm_color: "31", name: "Xen" },
    "uml" => VirtualizationInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "User-mode Linux" },
    "vmware" => VirtualizationInfo { icon: "vm", color: "#78A935", cterm_color: "106", name: "VMware" },
    "oracle" => VirtualizationInfo { icon: "󰐪", color: "#F80000", cterm_color: "196", name: "Oracle VM / VirtualBox" },
    "microsoft" => VirtualizationInfo { icon: "", color: "#00A4EF", cterm_color: "39", name: "Microsoft Hyper-V / Azure" },
    "zvm" => VirtualizationInfo { icon: "", color: "#0066CC", cterm_color: "26", name: "IBM z/VM" },
    "parallels" => VirtualizationInfo { icon: "||", color: "#DE0028", cterm_color: "160", name: "Parallels" },
    "bhyve" => VirtualizationInfo { icon: "≡", color: "#AB2B28", cterm_color: "124", name: "FreeBSD bhyve" },
    "qnx" => VirtualizationInfo { icon: "", color: "#000000", cterm_color: "232", name: "QNX Hypervisor" },
    "acrn" => VirtualizationInfo { icon: "", color: "#0071C5", cterm_color: "26", name: "Project ACRN" },
    "powervm" => VirtualizationInfo { icon: "", color: "#5486C3", cterm_color: "68", name: "IBM PowerVM" },
    "apple" => VirtualizationInfo { icon: "", color: "#A2AAAD", cterm_color: "248", name: "Apple VT" },
    "sre" => VirtualizationInfo { icon: "", color: "#4285F4", cterm_color: "33", name: "Google SRE" },
    "google" => VirtualizationInfo { icon: "", color: "#4285F4", cterm_color: "33", name: "Google Cloud Platform" },
    "vm-other" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Unknown VM" },

    // ===================================================================
    // Containers / OS-Level Virtualization
    // ===================================================================

    "systemd-nspawn" => VirtualizationInfo { icon: "", color: "#EE0000", cterm_color: "196", name: "systemd-nspawn" },
    "lxc-libvirt" => VirtualizationInfo { icon: "", color: "#F7931E", cterm_color: "208", name: "LXC (libvirt)" },
    "lxc" => VirtualizationInfo { icon: "", color: "#F7931E", cterm_color: "208", name: "LXC" },
    "openvz" => VirtualizationInfo { icon: "", color: "#00AEEF", cterm_color: "39", name: "OpenVZ" },
    "docker" => VirtualizationInfo { icon: "", color: "#2496ED", cterm_color: "39", name: "Docker" },
    "podman" => VirtualizationInfo { icon: "", color: "#892CA0", cterm_color: "98", name: "Podman" },
    "rkt" => VirtualizationInfo { icon: "", color: "#AC1A00", cterm_color: "124", name: "rkt" },
    "wsl" => VirtualizationInfo { icon: "", color: "#00A4EF", cterm_color: "39", name: "WSL" },
    "proot" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "proot" },
    "pouch" => VirtualizationInfo { icon: "", color: "#E67924", cterm_color: "208", name: "PouchContainer" },
    "container-other" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Unknown Container" },
};

/// Fast-path detection of common containers, WSL, and hypervisors without spawning a subprocess.
fn detect_virt_fast() -> Option<SmolStr> {
    if Path::new("/run/.containerenv").exists() {
        return Some(SmolStr::new_static("podman"));
    }
    if Path::new("/.dockerenv").exists() {
        return Some(SmolStr::new_static("docker"));
    }
    if let Ok(c) = std::env::var("container") {
        let c = c.trim();
        if !c.is_empty() {
            return Some(SmolStr::new(c));
        }
    }
    if Path::new("/proc/vz").exists() && !Path::new("/proc/bc").exists() {
        return Some(SmolStr::new_static("openvz"));
    }
    if std::env::var_os("WSL_DISTRO_NAME").is_some() || std::env::var_os("WSL_INTEROP").is_some() {
        return Some(SmolStr::new_static("wsl"));
    }
    if let Ok(osrelease) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let lower = osrelease.to_ascii_lowercase();
        if lower.contains("microsoft") || lower.contains("wsl") {
            return Some(SmolStr::new_static("wsl"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(present) = crate::sys::macos_hv_vmm_present() {
            return Some(SmolStr::new_static(if present { "apple" } else { "none" }));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(hv) = detect_linux_hypervisor_sysfs() {
            return Some(SmolStr::new_static(hv));
        }
    }

    None
}

#[cfg(any(target_os = "linux", test))]
fn classify_dmi_strings(texts: &[&str]) -> Option<&'static str> {
    for raw in texts {
        let s = raw.to_ascii_lowercase();
        if s.contains("amazon ec2") {
            return Some("amazon");
        }
        if s.contains("google compute engine") || s == "google" {
            return Some("google");
        }
        if s.contains("vmware") {
            return Some("vmware");
        }
        if s.contains("virtualbox") || s.contains("innotek") {
            return Some("oracle");
        }
        if s.contains("parallels") {
            return Some("parallels");
        }
        if s.contains("bhyve") {
            return Some("bhyve");
        }
        if s.contains("xen") {
            return Some("xen");
        }
        if s.contains("kvm") {
            return Some("kvm");
        }
        if s.contains("qemu") {
            return Some("qemu");
        }
        if s.contains("bochs") {
            return Some("bochs");
        }
        if s.contains("apple virtualization") {
            return Some("apple");
        }
        if s.contains("virtual machine") && s.contains("microsoft") {
            return Some("microsoft");
        }
    }

    // Separate vendor="Microsoft Corporation" + product="Virtual Machine" across DMI files.
    let has_ms = texts
        .iter()
        .any(|t| t.to_ascii_lowercase().contains("microsoft"));
    let has_vm = texts
        .iter()
        .any(|t| t.to_ascii_lowercase().contains("virtual machine"));
    if has_ms && has_vm {
        return Some("microsoft");
    }

    None
}

#[cfg(target_os = "linux")]
fn detect_linux_hypervisor_sysfs() -> Option<&'static str> {
    if Path::new("/proc/xen").exists() {
        return Some("xen");
    }
    if let Ok(hv_type) = std::fs::read_to_string("/sys/hypervisor/type") {
        if hv_type.trim().eq_ignore_ascii_case("xen") {
            return Some("xen");
        }
    }
    if let Ok(compat) = std::fs::read_to_string("/proc/device-tree/hypervisor/compatible") {
        let lower = compat.to_ascii_lowercase();
        if lower.contains("kvm") {
            return Some("kvm");
        }
        if lower.contains("xen") {
            return Some("xen");
        }
        if lower.contains("vmware") {
            return Some("vmware");
        }
    }

    let mut dmi_values = smallvec::SmallVec::<[String; 4]>::new();
    for path in [
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/board_vendor",
        "/sys/class/dmi/id/bios_vendor",
    ] {
        if let Ok(v) = std::fs::read_to_string(path) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                dmi_values.push(trimmed.to_string());
            }
        }
    }

    let refs: smallvec::SmallVec<[&str; 4]> = dmi_values.iter().map(String::as_str).collect();
    if let Some(hv) = classify_dmi_strings(&refs) {
        return Some(hv);
    }

    // Bare-metal x86/x86_64 fast-path: if DMI exists, matched no hypervisor, and CPU flags
    // do not advertise `hypervisor`, skip spawning `systemd-detect-virt`.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !dmi_values.is_empty() {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            let has_flags = cpuinfo.lines().any(|l| l.starts_with("flags"));
            let has_hv_flag = cpuinfo
                .lines()
                .filter(|l| l.starts_with("flags"))
                .any(|l| l.split_whitespace().any(|tok| tok == "hypervisor"));
            if has_flags && !has_hv_flag {
                return Some("none");
            }
        }
    }

    None
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.virt {
        return None;
    }

    let virt = match detect_virt_fast() {
        Some(v) => v,
        None => {
            CMD.exec::<_, &'static str>("systemd-detect-virt", [])
                .await?
        }
    };

    let icon = match virt {
        ref v if v == "none" => None, // No virtualization detected; return None
        ref v => VIRTUALIZATION_MAP.get(v.as_str()).map(|info| info.icon),
    };

    icon.map(|icon| {
        Chunk::info(if icon.is_empty() {
            virt
        } else {
            opts.select_str(&virt, icon).to_smolstr()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::classify_dmi_strings;

    #[test]
    fn test_classify_dmi_strings() {
        assert_eq!(
            classify_dmi_strings(&["Amazon EC2", "t4g.medium"]),
            Some("amazon")
        );
        assert_eq!(
            classify_dmi_strings(&["Google Compute Engine", "Google"]),
            Some("google")
        );
        assert_eq!(
            classify_dmi_strings(&["Virtual Machine", "Microsoft Corporation"]),
            Some("microsoft")
        );
        assert_eq!(
            classify_dmi_strings(&["Standard PC (Q35 + ICH9, 2009)", "QEMU"]),
            Some("qemu")
        );
        assert_eq!(
            classify_dmi_strings(&["ThinkPad X1 Carbon", "LENOVO"]),
            None
        );
    }
}
