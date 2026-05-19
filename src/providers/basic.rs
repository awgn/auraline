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
        .then(|| std::env::var("USER").ok().or_else(|| whoami::fallible::username().ok()).map(Chunk::info))
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
pub async fn pwd(opts: &Options) -> Option<Chunk<std::borrow::Cow<'static, str>>> {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| std::env::home_dir());
        
    opts.pwd
        .then(|| {
            let current_dir = std::env::var("PWD")
                .map(std::path::PathBuf::from)
                .ok()
                .or_else(|| std::env::current_dir().ok());

            current_dir.map(|p| {
                let pwd = if let Some(home) = &home {
                    if let Ok(stripped) = p.strip_prefix(home) {
                        if stripped.as_os_str().is_empty() {
                            std::borrow::Cow::Borrowed("~")
                        } else {
                            std::borrow::Cow::Owned(format!(
                                "~{}{}",
                                std::path::MAIN_SEPARATOR,
                                stripped.display()
                            ))
                        }
                    } else {
                        std::borrow::Cow::Owned(p.to_string_lossy().into_owned())
                    }
                } else {
                    std::borrow::Cow::Owned(p.to_string_lossy().into_owned())
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
            std::env::var("PWD")
                .ok()
                .or_else(|| {
                    std::env::current_dir()
                        .ok()
                        .map(|pwd| pwd.to_string_lossy().into_owned())
                })
                .map(Chunk::info)
        })
        .flatten()
}
