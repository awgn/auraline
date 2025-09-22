use smol_str::{format_smolstr, SmolStr};

use crate::{cmd::CMD, options::Options};

pub async fn namespace(_: &Options) -> Option<SmolStr> {
    CMD.exec("ip", ["netns", "identify"])
        .await
        .filter(|s| !s.is_empty())
        .map(|s| format_smolstr!("⁅{s}"))
}
