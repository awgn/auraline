use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{Arc, Mutex},
    task::Poll,
};
use tokio::process::Command;

lazy_static! {
    pub static ref CMD: CmdCache = CmdCache::new();
}

#[derive(Debug, Clone)]
struct CmdOutput(Arc<tokio::sync::Mutex<Poll<Option<String>>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CmdKey(&'static str, Vec<String>);

pub struct CmdCache {
    cache: Mutex<HashMap<CmdKey, CmdOutput>>,
}

impl CmdCache {
    fn new() -> Self {
        Self {
            cache: Mutex::new(std::collections::HashMap::new()),
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
                .map(|s| s.as_ref().to_os_string().into_string().unwrap())
                .collect::<Vec<_>>(),
        )
    }

    pub async fn exec<I, S>(&self, cmd: &'static str, args: I) -> Option<String>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        let key = Self::make_key(cmd, args.clone());
        let value = {
            let mut cache = self.cache.lock().unwrap();
            let value = cache
                .entry(key)
                .or_insert_with(|| CmdOutput(Arc::new(tokio::sync::Mutex::new(Poll::Pending))));
            value.clone()
        };

        let mut value = value.0.lock().await;
        match *value {
            Poll::Ready(ref v) => v.clone(),
            Poll::Pending => {
                let output = {
                    let output = Command::new(cmd).args(args).output().await.ok()?;

                    // tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                    if output.status.success() {
                        Some(
                            String::from_utf8(output.stdout)
                                .unwrap()
                                .trim_end()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                };
                *value = Poll::Ready(output.clone());
                output
            }
        }
    }
}
