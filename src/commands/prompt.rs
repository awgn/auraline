use frunk::hlist;
use frunk::Func;
use frunk::HCons;
use frunk::HNil;

use frunk::Poly;
use std::env;
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinError;

use crate::providers::basic::device_name;
use crate::providers::basic::distro;
use crate::providers::basic::full_pwd;
use crate::providers::basic::hostname;
use crate::providers::basic::pwd;
use crate::providers::basic::realname;
use crate::providers::basic::user;
use crate::providers::duration::show as duration_show;
use crate::providers::exit_code::show as exit_code_show;

use crate::chunk::Chunk;
use crate::providers::huge_pages::show as huge_pages_show;
use crate::providers::manifest::show as manifest_show;
use crate::providers::memory::show as memory_show;
use crate::providers::netif::show as netif_show;
use crate::providers::netns::show as net_namespace;
use crate::providers::os::show as os_show;
use crate::providers::ssh::show as ssh_show;
use crate::providers::vcs::{infer_vcs, Vcs, VcsTrait};
use crate::providers::virt::show as virt_show;

use crate::style::build_color_style;

use crate::Options;
use owo_colors::Style;

macro_rules! item {
    ($provider:expr, $opt:expr, $style:expr) => {{
        let cloned_opts = Arc::clone(&$opt);
        let style = $style;
        tokio::spawn(async move {
            let begin = std::time::Instant::now();
            let res = $provider(&cloned_opts)
                .await
                .map(|c| c.with_style(style.0, style.1));
            (provider_name(&$provider), begin.elapsed(), res)
        })
    }};
}

macro_rules! item_vcs {
    ($vcs:expr, $provider:expr, $opt:expr, $style:expr) => {{
        let cloned_opts = Arc::clone(&$opt);
        let vcs = $vcs.clone();
        let style = $style;
        tokio::spawn(async move {
            let begin = std::time::Instant::now();

            if let Some((vcs, path)) = vcs {
                let res = $provider(&vcs, &cloned_opts, &path)
                    .await
                    .map(|c| c.with_style(style.0, style.1));
                return (provider_name(&$provider), begin.elapsed(), res);
            }

            (provider_name(&$provider), begin.elapsed(), None)
        })
    }};
}

pub async fn print_prompt(opts: Options) -> anyhow::Result<()> {
    let start = if opts.timings {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let opts = Arc::new(opts);
    let cwd = env::current_dir()?;
    let vcs = infer_vcs(cwd, &opts).await;

    let (color, bold, def) = (
        build_color_style(opts.theme.as_deref()),
        Style::new().bold(),
        Style::default(),
    );

    let async_prompt = hlist![
        item![user, opts, (color, bold)],
        item![realname, opts, (color, bold)],
        item![hostname, opts, (color, bold.dimmed())],
        item![device_name, opts, (color, bold.dimmed())],
        item![distro, opts, (color, bold.dimmed())],
        item![pwd, opts, (color, bold)],
        item![full_pwd, opts, (color, bold)],
        item![os_show, opts, (color, bold)],
        item![virt_show, opts, (bold, bold)],
        item![memory_show, opts, (bold, bold)],
        item![huge_pages_show, opts, (bold, bold)],
        item![ssh_show, opts, (bold, def)],
        item![netif_show, opts, (bold.dimmed(), def.dimmed())],
        item![net_namespace, opts, (bold, bold)],
        item![manifest_show, opts, (color, color)],
        item_vcs![vcs, <Vcs as VcsTrait>::branch, opts, (bold, color.bold())],
        item_vcs![vcs, <Vcs as VcsTrait>::status, opts, (bold, color)],
        item_vcs![vcs, <Vcs as VcsTrait>::stash, opts, (bold, def)],
        item_vcs![
            vcs,
            <Vcs as VcsTrait>::worktree,
            opts,
            (bold, bold.dimmed())
        ],
        item_vcs![vcs, <Vcs as VcsTrait>::commit, opts, (bold, bold)],
        item_vcs![vcs, <Vcs as VcsTrait>::divergence, opts, (bold, def)],
        item![duration_show, opts, (def, def.dimmed())],
        item![exit_code_show, opts, (bold.red(), bold)],
    ];

    let prompt = async_prompt.hjoin().await;

    if let Some(start) = start {
        prompt.map(Poly(TimingMapper));
        println!("{:<40} -> {:>15?}", "total time", start.elapsed());
    } else {
        prompt.map(Poly(PrintMapper));
    }

    Ok(())
}

trait HListJoin {
    type Output;
    async fn hjoin(self) -> Self::Output;
}

impl HListJoin for HNil {
    type Output = HNil;
    async fn hjoin(self) -> Self::Output {
        HNil
    }
}

impl<H, T> HListJoin for HCons<H, T>
where
    H: Future + Send,
    T: HListJoin + Send,
    T::Output: Send,
{
    type Output = HCons<H::Output, T::Output>;
    async fn hjoin(self) -> Self::Output {
        let (head_res, tail_res) = tokio::join!(self.head, self.tail.hjoin());
        HCons {
            head: head_res,
            tail: tail_res,
        }
    }
}

struct PrintMapper;
impl<T> Func<Result<(&'static str, Duration, Option<Chunk<T>>), JoinError>> for PrintMapper
where
    T: Display,
{
    type Output = ();

    fn call(input: Result<(&'static str, Duration, Option<Chunk<T>>), JoinError>) -> Self::Output {
        if let (_, _, Some(c)) = input.expect("Task panicked") {
            print!("{c} ")
        }
    }
}

struct TimingMapper;
impl<T> Func<Result<(&'static str, Duration, Option<Chunk<T>>), JoinError>> for TimingMapper
where
    T: Display,
{
    type Output = ();

    fn call(input: Result<(&'static str, Duration, Option<Chunk<T>>), JoinError>) -> Self::Output {
        let (f, dur, c) = input.expect("Task panicked");
        let f = f.replace("auraline::providers::", "");
        if let Some(chunk) = c {
            println!("{f:<40} -> {dur:>15?} : ({chunk})");
        } else {
            println!("{f:<40} -> {dur:>15?} : (_)");
        }
    }
}
#[inline]
fn provider_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}
