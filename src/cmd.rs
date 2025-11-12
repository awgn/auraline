use lazy_static::lazy_static;
use parking_lot::Mutex;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{collections::HashMap, ffi::OsStr, sync::Arc, task::Poll};
use tokio::process::Command;

lazy_static! {
    pub static ref CMD: CmdCache = CmdCache::new();
}

#[derive(Debug, Clone)]
struct CmdOutput(Arc<tokio::sync::Mutex<Poll<Option<SmolStr>>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct CmdKey(&'static str, SmallVec<[SmolStr; 4]>);

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
        let value = {
            let mut cache = self.cache.lock();
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
                    if output.status.success() {
                        unsafe {
                            Some(SmolStr::new(
                                std::str::from_utf8_unchecked(&output.stdout).trim_end(),
                            ))
                        }
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
