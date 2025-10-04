use crate::{
    chunk::{Adjoin, Chunk},
    options::Options,
};

#[inline]
pub async fn hostname(opts: &Options) -> Option<Chunk<Adjoin<&'static str, String>>> {
    opts.hostname
        .then(|| {
            whoami::fallible::hostname()
                .ok()
                .map(|h| Chunk::info(Adjoin(("@", h))))
        })
        .flatten()
}

#[inline]
pub async fn device_name(opts: &Options) -> Option<Chunk<String>> {
    opts.device_name
        .then(|| {
            whoami::fallible::devicename()
                .ok()
                .map(|d| Chunk::new("🖥", d))
        })
        .flatten()
}

#[inline]
pub async fn user(opts: &Options) -> Option<Chunk<String>> {
    opts.user
        .then(|| whoami::fallible::username().ok().map(Chunk::info))
        .flatten()
}

#[inline]
pub async fn realname(opts: &Options) -> Option<Chunk<String>> {
    opts.realname
        .then(|| whoami::fallible::realname().ok().map(Chunk::info))
        .flatten()
}

#[inline]
pub async fn distro(opts: &Options) -> Option<Chunk<String>> {
    opts.distro
        .then(|| whoami::fallible::distro().ok().map(Chunk::info))
        .flatten()
}

#[inline]
pub async fn pwd(opts: &Options) -> Option<Chunk<String>> {
    let home = std::env::home_dir();
    opts.pwd
        .then(|| {
            std::env::current_dir().ok().map(|p| {
                let pwd = if let Some(home) = &home {
                    if let Ok(stripped) = p.strip_prefix(home) {
                        if stripped.as_os_str().is_empty() {
                            "~".to_string()
                        } else {
                            format!(
                                "~{}",
                                std::path::MAIN_SEPARATOR.to_string().to_owned()
                                    + &stripped.to_string_lossy()
                            )
                        }
                    } else {
                        p.to_string_lossy().to_string()
                    }
                } else {
                    p.to_string_lossy().to_string()
                };

                Chunk::info(pwd)
            })
        })
        .flatten()
}

#[inline]
pub async fn full_pwd(opts: &Options) -> Option<Chunk<String>> {
    opts.full_pwd
        .then(|| {
            std::env::current_dir()
                .ok()
                .map(|pwd| Chunk::info(pwd.to_string_lossy().to_string()))
        })
        .flatten()
}
