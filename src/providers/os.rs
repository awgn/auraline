use phf::phf_map;

use crate::{
    chunk::{Chunk, Unit},
    commands::Options,
};

#[allow(dead_code)]
pub struct OsInfo {
    pub icon: &'static str,
    pub color: &'static str,
    pub cterm_color: &'static str,
    pub name: &'static str,
}

// this map is expired to https://github.com/nvim-tree/nvim-web-devicons
//
static OS_MAP: phf::Map<&'static str, OsInfo> = phf_map! {
    "alma" => OsInfo { icon: "", color: "#FF4649", cterm_color: "203", name: "Almalinux" },
    "alpine" => OsInfo { icon: "", color: "#0D597F", cterm_color: "24", name: "Alpine" },
    "aosc" => OsInfo { icon: "", color: "#C00000", cterm_color: "124", name: "AOSC" },
    "apple" => OsInfo { icon: "", color: "#A2AAAD", cterm_color: "248", name: "Apple" },
    "arch" => OsInfo { icon: "󰣇", color: "#0F94D2", cterm_color: "67", name: "Arch" },
    "archcraft" => OsInfo { icon: "", color: "#86BBA3", cterm_color: "108", name: "Archcraft" },
    "archlabs" => OsInfo { icon: "", color: "#503F42", cterm_color: "238", name: "Archlabs" },
    "arcolinux" => OsInfo { icon: "", color: "#6690EB", cterm_color: "68", name: "ArcoLinux" },
    "artix" => OsInfo { icon: "", color: "#41B4D7", cterm_color: "38", name: "Artix" },
    "biglinux" => OsInfo { icon: "", color: "#189FC8", cterm_color: "38", name: "BigLinux" },
    "centos" => OsInfo { icon: "", color: "#A2518D", cterm_color: "132", name: "Centos" },
    "crystallinux" => OsInfo { icon: "", color: "#A900FF", cterm_color: "129", name: "CrystalLinux" },
    "debian" => OsInfo { icon: "", color: "#A80030", cterm_color: "88", name: "Debian" },
    "deepin" => OsInfo { icon: "", color: "#2CA7F8", cterm_color: "39", name: "Deepin" },
    "devuan" => OsInfo { icon: "", color: "#404A52", cterm_color: "238", name: "Devuan" },
    "elementary" => OsInfo { icon: "", color: "#5890C2", cterm_color: "67", name: "Elementary" },
    "endeavour" => OsInfo { icon: "", color: "#7B3DB9", cterm_color: "91", name: "Endeavour" },
    "fedora" => OsInfo { icon: "", color: "#072A5E", cterm_color: "17", name: "Fedora" },
    "freebsd" => OsInfo { icon: "", color: "#C90F02", cterm_color: "160", name: "FreeBSD" },
    "netbsd" => OsInfo { icon: "⚑", color: "#FF6600", cterm_color: "208", name: "NetBSD" },
    "garuda" => OsInfo { icon: "", color: "#2974E1", cterm_color: "33", name: "GarudaLinux" },
    "gentoo" => OsInfo { icon: "󰣨", color: "#B1ABCE", cterm_color: "146", name: "Gentoo" },
    "guix" => OsInfo { icon: "", color: "#FFCC00", cterm_color: "220", name: "Guix" },
    "hyperbola" => OsInfo { icon: "", color: "#C0C0C0", cterm_color: "250", name: "HyperbolaGNULinuxLibre" },
    "illumos" => OsInfo { icon: "", color: "#FF430F", cterm_color: "196", name: "Illumos" },
    "kali" => OsInfo { icon: "", color: "#2777FF", cterm_color: "69", name: "Kali" },
    "kdeneon" => OsInfo { icon: "", color: "#20A6A4", cterm_color: "37", name: "KDEneon" },
    "kubuntu" => OsInfo { icon: "", color: "#007AC2", cterm_color: "32", name: "Kubuntu" },
    "leap" => OsInfo { icon: "", color: "#FBC75D", cterm_color: "221", name: "Leap" },
    "linux" => OsInfo { icon: "", color: "#FDFDFB", cterm_color: "231", name: "Linux" },
    "locos" => OsInfo { icon: "", color: "#FAB402", cterm_color: "214", name: "LocOS" },
    "lxle" => OsInfo { icon: "", color: "#474747", cterm_color: "238", name: "LXLE" },
    "mageia" => OsInfo { icon: "", color: "#2397D4", cterm_color: "67", name: "Mageia" },
    "manjaro" => OsInfo { icon: "", color: "#33B959", cterm_color: "35", name: "Manjaro" },
    "mint" => OsInfo { icon: "󰣭", color: "#66AF3D", cterm_color: "70", name: "Mint" },
    "mxlinux" => OsInfo { icon: "", color: "#FFFFFF", cterm_color: "231", name: "MXLinux" },
    "nixos" => OsInfo { icon: "", color: "#7AB1DB", cterm_color: "110", name: "NixOS" },
    "nobara" => OsInfo { icon: "", color: "#FFFFFF", cterm_color: "231", name: "NobaraLinux" },
    "openbsd" => OsInfo { icon: "", color: "#F2CA30", cterm_color: "220", name: "OpenBSD" },
    "opensuse" => OsInfo { icon: "", color: "#6FB424", cterm_color: "70", name: "openSUSE" },
    "parabola" => OsInfo { icon: "", color: "#797DAC", cterm_color: "103", name: "ParabolaGNULinuxLibre" },
    "parrot" => OsInfo { icon: "", color: "#54DEFF", cterm_color: "45", name: "Parrot" },
    "pop_os" => OsInfo { icon: "", color: "#48B9C7", cterm_color: "73", name: "Pop_OS" },
    "postmarketos" => OsInfo { icon: "", color: "#009900", cterm_color: "28", name: "postmarketOS" },
    "puppylinux" => OsInfo { icon: "", color: "#A2AEB9", cterm_color: "145", name: "PuppyLinux" },
    "qubesos" => OsInfo { icon: "", color: "#3774D8", cterm_color: "33", name: "QubesOS" },
    "raspberry_pi" => OsInfo { icon: "", color: "#BE1848", cterm_color: "161", name: "RaspberryPiOS" },
    "redhat" => OsInfo { icon: "󱄛", color: "#EE0000", cterm_color: "196", name: "Redhat" },
    "rocky" => OsInfo { icon: "", color: "#0FB37D", cterm_color: "36", name: "RockyLinux" },
    "sabayon" => OsInfo { icon: "", color: "#C6C6C6", cterm_color: "251", name: "Sabayon" },
    "slackware" => OsInfo { icon: "", color: "#475FA9", cterm_color: "61", name: "Slackware" },
    "solus" => OsInfo { icon: "", color: "#4B5163", cterm_color: "239", name: "Solus" },
    "tails" => OsInfo { icon: "", color: "#56347C", cterm_color: "54", name: "Tails" },
    "trisquel" => OsInfo { icon: "", color: "#0F58B6", cterm_color: "25", name: "TrisquelGNULinux" },
    "tumbleweed" => OsInfo { icon: "", color: "#35B9AB", cterm_color: "37", name: "Tumbleweed" },
    "ubuntu" => OsInfo { icon: "", color: "#DD4814", cterm_color: "196", name: "Ubuntu" },
    "vanillaos" => OsInfo { icon: "", color: "#FABD4D", cterm_color: "214", name: "VanillaOS" },
    "void" => OsInfo { icon: "", color: "#295340", cterm_color: "23", name: "Void" },
    "windows" => OsInfo { icon: "", color: "#00A4EF", cterm_color: "39", name: "Windows" },
    "xerolinux" => OsInfo { icon: "", color: "#888FE2", cterm_color: "104", name: "XeroLinux" },
    "zorin" => OsInfo { icon: "", color: "#14A1E8", cterm_color: "39", name: "Zorin" },
};

// see few examples @ https://gist.github.com/natefoo/814c5bf936922dad97ff
pub async fn lsb_icon() -> Option<Chunk<Unit>> {
    let os_release = tokio::fs::read_to_string("/etc/os-release").await.ok()?;
    let id_line = os_release.lines().find(|line| line.starts_with("ID="))?;
    let id = id_line.trim_start_matches("ID=").trim_matches('"');
    OS_MAP
        .get(id)
        .map(|info| Chunk::icon(info.icon))
        .or_else(|| OS_MAP.get("linux").map(|info| Chunk::icon(info.icon)))
}

pub async fn show(opts: &Options) -> Option<Chunk<Unit>> {
    if !opts.nerd_font {
        return None;
    }
    match std::env::consts::OS {
        "linux" => lsb_icon().await,
        "windows" => OS_MAP.get("windows").map(|info| Chunk::icon(info.icon)),
        "macos" => OS_MAP.get("apple").map(|info| Chunk::icon(info.icon)),
        "ios" => OS_MAP.get("apple").map(|info| Chunk::icon(info.icon)),
        "openbsd" => OS_MAP.get("openbsd").map(|info| Chunk::icon(info.icon)),
        "freebsd" => OS_MAP.get("freebsd").map(|info| Chunk::icon(info.icon)),
        "netbsd" => OS_MAP.get("netbsd").map(|info| Chunk::icon(info.icon)),
        "apple" => OS_MAP.get("apple").map(|info| Chunk::icon(info.icon)),
        "illumos" => OS_MAP.get("illumos").map(|info| Chunk::icon(info.icon)),
        _ => None,
    }
}
