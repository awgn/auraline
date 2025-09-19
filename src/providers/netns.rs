use crate::cmd::CMD;

pub async fn net_namespace() -> Option<String> {
    CMD.exec("ip", ["netns", "identify"])
        .await
        .filter(|s| !s.is_empty())
        .map(|s| format!("⁅{s}"))
}
