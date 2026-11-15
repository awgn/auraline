use crate::{
    chunk::{Chunk, Unit},
    cmd::CMD,
    options::Options,
};

#[cfg(unix)]
#[inline]
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(unix))]
#[inline]
fn is_root() -> bool {
    false
}

pub async fn show(opts: &Options) -> Option<Chunk<Unit>> {
    if !opts.sudo {
        return None;
    }

    // `sudo -Nnv` checks cached credentials without updating/extending timestamp_timeout.
    let has_sudo = is_root() || CMD.exec("sudo", ["-N", "-n", "-v"]).await.is_some();

    has_sudo.then(|| Chunk::icon(opts.select_str("↯", "󰌋")))
}
