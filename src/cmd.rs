use papaya::HashMap;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::process::Command;
use tokio::sync::OnceCell;

/// Defensive timeout preventing hung external tools or wedged network mounts
/// from freezing interactive shell prompt rendering.
const CMD_TIMEOUT: Duration = Duration::from_millis(500);

pub static CMD: LazyLock<CmdCache> = LazyLock::new(CmdCache::new);
static PATH_CACHE: LazyLock<HashMap<&'static str, Option<PathBuf>>> = LazyLock::new(HashMap::new);

#[derive(Debug, Clone)]
struct CmdOutput(Arc<OnceCell<Option<SmolStr>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CmdKey(Option<PathBuf>, &'static str, SmallVec<[SmolStr; 4]>);

pub struct CmdCache {
    cache: HashMap<CmdKey, CmdOutput>,
}

/// Resolves `cmd` to an executable `PathBuf` via `PATH`, caching the result
/// so each unique command name scans `PATH` at most once per process.
pub fn resolve_in_path(cmd: &'static str) -> Option<PathBuf> {
    let pinned = PATH_CACHE.pin();
    pinned
        .get_or_insert_with(cmd, || {
            if cmd.contains(std::path::MAIN_SEPARATOR) {
                let p = PathBuf::from(cmd);
                p.is_file().then_some(p)
            } else {
                std::env::var_os("PATH").and_then(|path_var| {
                    std::env::split_paths(&path_var)
                        .map(|dir| dir.join(cmd))
                        .find(|candidate| candidate.is_file())
                })
            }
        })
        .clone()
}

impl CmdCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn make_key<I, S>(cwd: Option<&Path>, cmd: &'static str, args: I) -> CmdKey
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        CmdKey(
            cwd.map(Path::to_path_buf),
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
        self.exec_in(None, cmd, args).await
    }

    pub async fn exec_in<I, S>(
        &self,
        cwd: Option<&Path>,
        cmd: &'static str,
        args: I,
    ) -> Option<SmolStr>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let resolved_bin = resolve_in_path(cmd);
        let key = Self::make_key(cwd, cmd, args.clone());
        let cell = {
            let pinned = self.cache.pin();
            pinned
                .get_or_insert_with(key, || {
                    let cell = Arc::new(OnceCell::new());
                    // If the binary doesn't exist in PATH pre-populate with None,
                    // so get_or_init returns immediately without forking.
                    if resolved_bin.is_none() {
                        cell.set(None).ok();
                    }
                    CmdOutput(cell)
                })
                .clone()
        };

        let cwd_owned = cwd.map(Path::to_path_buf);

        // OnceCell guarantees the init closure runs exactly once even under
        // concurrent access: the first caller executes the command while the
        // others wait — without holding any lock.
        cell.0
            .get_or_init(|| async move {
                let bin = resolved_bin?;
                let mut command = Command::new(bin);
                command.args(args).kill_on_drop(true);
                if let Some(dir) = cwd_owned {
                    command.current_dir(dir);
                }
                let output = tokio::time::timeout(CMD_TIMEOUT, command.output())
                    .await
                    .ok()?
                    .ok()?;
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    Some(SmolStr::new(text.trim_end()))
                } else {
                    None
                }
            })
            .await
            .clone()
    }
}
