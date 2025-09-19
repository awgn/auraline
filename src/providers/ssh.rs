pub async fn show() -> Option<String> {
    let ssh_connection = std::env::var("SSH_CONNECTION").ok()?;
    let parts: Vec<&str> = ssh_connection.split_whitespace().collect();
    if parts.len() >= 4 {
        Some(format!("⇄{}:{}", parts[2], parts[3]))
    } else {
        None
    }
}
