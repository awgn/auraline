use lazy_static::lazy_static;
use papaya::HashMap;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::process::Command;
use tokio::sync::OnceCell;

lazy_static! {
    pub static ref CMD: CmdCache = CmdCache::new();
    static ref PATH_CACHE: HashMap<&'static str, bool> = HashMap::new();
}

#[derive(Debug, Clone)]
struct CmdOutput(Arc<OnceCell<Option<SmolStr>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CmdKey(Option<PathBuf>, &'static str, SmallVec<[SmolStr; 4]>);

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
        let key = Self::make_key(cwd, cmd, args.clone());
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

        let cwd_owned = cwd.map(Path::to_path_buf);

        // OnceCell guarantees the init closure runs exactly once even under
        // concurrent access: the first caller executes the command while the
        // others wait — without holding any lock.
        cell.0
            .get_or_init(|| async move {
                let mut command = Command::new(cmd);
                command.args(args);
                if let Some(dir) = cwd_owned {
                    command.current_dir(dir);
                }
                let output = command.output().await.ok()?;
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
