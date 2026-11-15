use crate::{chunk::Chunk, options::Options, sys};
use smol_str::{SmolStr, SmolStrBuilder};

#[inline]
pub async fn hostname(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.hostname {
        return None;
    }
    sys::hostname().map(|h| Chunk::adjoin("@", h))
}

#[inline]
pub async fn device_name(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.device_name {
        return None;
    }
    sys::device_name()
        .await
        .map(|d| Chunk::new(opts.select_str("⎚", "󰍹"), d))
}

#[inline]
pub async fn user(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.user {
        return None;
    }
    sys::username().map(Chunk::info)
}

#[inline]
pub async fn realname(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.realname || (opts.user && sys::username().is_some()) {
        return None;
    }
    sys::realname().map(Chunk::info)
}

#[inline]
pub async fn distro(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.distro {
        return None;
    }
    sys::distro().map(Chunk::info)
}

#[inline]
pub async fn pwd(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.pwd || opts.full_pwd {
        return None;
    }

    let home = std::env::var("HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(std::env::home_dir);

    let current_dir = std::env::var("PWD")
        .ok()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;

    let pwd = if let Some(home) = &home {
        if let Ok(stripped) = current_dir.strip_prefix(home) {
            if stripped.as_os_str().is_empty() {
                SmolStr::new_static("~")
            } else {
                let mut builder = SmolStrBuilder::new();
                builder.push('~');
                builder.push(std::path::MAIN_SEPARATOR);
                builder.push_str(&stripped.to_string_lossy());
                builder.finish()
            }
        } else {
            SmolStr::new(current_dir.to_string_lossy().as_ref())
        }
    } else {
        SmolStr::new(current_dir.to_string_lossy().as_ref())
    };

    Some(Chunk::info(pwd))
}

#[inline]
pub async fn full_pwd(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.full_pwd {
        return None;
    }

    std::env::var("PWD")
        .ok()
        .filter(|s| !s.is_empty())
        .map(SmolStr::from)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|pwd| SmolStr::new(pwd.to_string_lossy().as_ref()))
        })
        .map(Chunk::info)
}
