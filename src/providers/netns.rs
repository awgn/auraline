use crate::{cmd::CMD, options::Options};

pub async fn namespace(_: &Options) -> Option<String> {
    CMD.exec("ip", ["netns", "identify"])
        .await
        .filter(|s| !s.is_empty())
        .map(|s| format!("⁅{s}"))
}
