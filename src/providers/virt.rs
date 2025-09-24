use crate::{chunk::Chunk, cmd::CMD, options::Options};
use phf::phf_map;
use smol_str::{SmolStr, ToSmolStr};

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

    "systemd-nspawn" => VirtualizationInfo { icon: "", color: "#EE0000", cterm_color: "196", name: "systemd-nspawn" },
    "lxc-libvirt" => VirtualizationInfo { icon: "", color: "#F7931E", cterm_color: "208", name: "LXC (libvirt)" },
    "lxc" => VirtualizationInfo { icon: "", color: "#F7931E", cterm_color: "208", name: "LXC" },
    "openvz" => VirtualizationInfo { icon: "", color: "#00AEEF", cterm_color: "39", name: "OpenVZ" },
    "docker" => VirtualizationInfo { icon: "", color: "#2496ED", cterm_color: "39", name: "Docker" },
    "podman" => VirtualizationInfo { icon: "", color: "#892CA0", cterm_color: "98", name: "Podman" },
    "rkt" => VirtualizationInfo { icon: "", color: "#AC1A00", cterm_color: "124", name: "rkt" },
    "wsl" => VirtualizationInfo { icon: "", color: "#00A4EF", cterm_color: "39", name: "WSL" },
    "proot" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "proot" },
    "pouch" => VirtualizationInfo { icon: "", color: "#E67924", cterm_color: "208", name: "PouchContainer" },
    "container-other" => VirtualizationInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Unknown Container" },
};

pub async fn show(_: &Options) -> Option<Chunk<SmolStr>> {
    let virt = CMD
        .exec::<_, &'static str>("systemd-detect-virt", [])
        .await?;
    let icon = match virt {
        ref v if v == "none" => None, // No virtualization detected; return None
        ref v => VIRTUALIZATION_MAP.get(v).map(|info| info.icon),
    };

    icon.map(|ico| {
        Chunk::info(if ico.is_empty() {
            virt.to_smolstr()
        } else {
            ico.to_smolstr()
        })
    })
}
