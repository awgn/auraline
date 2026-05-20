use lazy_static::lazy_static;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{ffi::OsStr, path::Path, sync::Arc};
use tokio::process::Command;
use tokio::sync::OnceCell;
use papaya::HashMap;

lazy_static! {
    pub static ref CMD: CmdCache = CmdCache::new();
    static ref PATH_CACHE: HashMap<&'static str, bool> = HashMap::new();
}

#[derive(Debug, Clone)]
struct CmdOutput(Arc<OnceCell<Option<SmolStr>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CmdKey(&'static str, SmallVec<[SmolStr; 4]>);

pub struct CmdCache {
    cache: HashMap<CmdKey, CmdOutput>,
}

/// Checks whether `cmd` resolves to an executable file via `PATH`.
/// Entirely synchronous and allocation-light; the result is cached by the
/// caller so this runs at most once per unique command name.
fn cmd_in_path(cmd: &'static str) -> bool {
    let pinned = PATH_CACHE.pin();
    *pinned.get_or_insert_with(cmd, || {
        // Absolute / relative path — just stat it directly.
        if cmd.contains(std::path::MAIN_SEPARATOR) {
            Path::new(cmd).is_file()
        } else {
            std::env::var_os("PATH")
                .map(|path_var| std::env::split_paths(&path_var).any(|dir| dir.join(cmd).is_file()))
                .unwrap_or(false)
        }
    })
}

impl CmdCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn make_key<I, S>(cmd: &'static str, args: I) -> CmdKey
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        CmdKey(
            cmd,
            args.into_iter()
                .filter_map(|s| s.as_ref().to_str().map(Into::into))
                .collect::<SmallVec<[_; 4]>>(),
        )
    }

    pub async fn exec<I, S>(&self, cmd: &'static str, args: I) -> Option<SmolStr>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let key = Self::make_key(cmd, args.clone());
        let cell = {
            let pinned = self.cache.pin();
            pinned
                .get_or_insert_with(key, || {
                    let cell = Arc::new(OnceCell::new());
                    // If the binary doesn't exist in PATH pre-populate with None,
                    // so get_or_init returns immediately without forking.
                    if !cmd_in_path(cmd) {
                        cell.set(None).ok();
                    }
                    CmdOutput(cell)
                })
                .clone()
        };

        // OnceCell guarantees the init closure runs exactly once even under
        // concurrent access: the first caller executes the command while the
        // others wait — without holding any lock.
        cell.0
            .get_or_init(|| async move {
                let output = Command::new(cmd).args(args).output().await.ok()?;
                if output.status.success() {
                    unsafe {
                        Some(SmolStr::new(
                            std::str::from_utf8_unchecked(&output.stdout).trim_end(),
                        ))
                    }
                } else {
                    None
                }
            })
            .await
            .clone()
    }
}
