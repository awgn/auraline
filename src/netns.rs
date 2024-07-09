use crate::cmd::CMD;

pub async fn net_namespace() -> Option<String> {
    CMD.exec("ip", ["netns", "identify"]).await.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(format!("⁅{s}"))
        }
    })
}
