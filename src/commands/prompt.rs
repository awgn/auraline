use frunk::{hlist, HCons, HNil};
use owo_colors::Style;
use std::env;
use std::fmt::{Display, Write as _};
use std::future::Future;
use std::io::Write as _;
use std::time::Duration;

use crate::chunk::Chunk;
use crate::providers::basic::{device_name, distro, full_pwd, hostname, pwd, realname, user};
use crate::providers::disk::show as disk_show;
use crate::providers::duration::show as duration_show;
use crate::providers::exit_code::show as exit_code_show;
use crate::providers::huge_pages::show as huge_pages_show;
use crate::providers::jobs::show as jobs_show;
use crate::providers::k8s::show as k8s_show;
use crate::providers::load::show as load_show;
use crate::providers::manifest::show as manifest_show;
use crate::providers::memory::show as memory_show;
use crate::providers::netif::show as netif_show;
use crate::providers::netns::show as net_namespace;
use crate::providers::nix::show as nix_show;
use crate::providers::os::show as os_show;
use crate::providers::shlvl::show as shlvl_show;
use crate::providers::ssh::show as ssh_show;
use crate::providers::sudo::show as sudo_show;
use crate::providers::vcs::{infer_vcs, Vcs, VcsTrait};
use crate::providers::venv::show as venv_show;
use crate::providers::virt::show as virt_show;
use crate::style::build_color_style;
use crate::Options;

macro_rules! item {
    ($enabled:expr, $provider:expr, $opt:expr, $style:expr) => {{
        let opt_ref = $opt;
        let enabled = $enabled;
        let style = $style;
        async move {
            if !enabled {
                return (provider_name(&$provider), Duration::ZERO, None);
            }
            let begin = opt_ref.timings.then(std::time::Instant::now);
            let res = $provider(opt_ref)
                .await
                .map(|c| c.with_style(style.0, style.1, opt_ref.shell_mode()));
            let elapsed = begin.map_or(Duration::ZERO, |b| b.elapsed());
            (provider_name(&$provider), elapsed, res)
        }
    }};
}

macro_rules! item_vcs {
    ($vcs:expr, $provider:expr, $opt:expr, $style:expr) => {{
        let opt_ref = $opt;
        let vcs_ref = $vcs;
        let style = $style;
        async move {
            let Some((vcs, path)) = vcs_ref else {
                return (provider_name(&$provider), Duration::ZERO, None);
            };
            let begin = opt_ref.timings.then(std::time::Instant::now);
            let res = $provider(vcs, opt_ref, path)
                .await
                .map(|c| c.with_style(style.0, style.1, opt_ref.shell_mode()));
            let elapsed = begin.map_or(Duration::ZERO, |b| b.elapsed());
            (provider_name(&$provider), elapsed, res)
        }
    }};
}

pub async fn print_prompt(opts: Options) -> anyhow::Result<()> {
    let start = opts.timings.then(std::time::Instant::now);

    let (color, bold, def) = (
        build_color_style(opts.theme.as_deref()),
        Style::new().bold(),
        Style::default(),
    );

    let pre_vcs = hlist![
        item![opts.sudo, sudo_show, &opts, (bold.yellow(), bold)],
        item![opts.user, user, &opts, (color, bold)],
        item![opts.realname, realname, &opts, (color, bold)],
        item![opts.hostname, hostname, &opts, (color, bold.dimmed())],
        item![opts.device_name, device_name, &opts, (color, bold.dimmed())],
        item![opts.distro, distro, &opts, (color, bold.dimmed())],
        item![opts.pwd, pwd, &opts, (color, bold)],
        item![opts.full_pwd, full_pwd, &opts, (color, bold)],
        item![opts.shlvl, shlvl_show, &opts, (color, color.bold())],
        item![opts.os, os_show, &opts, (color, bold)],
        item![opts.virt, virt_show, &opts, (bold, bold)],
        item![opts.memory, memory_show, &opts, (bold, bold)],
        item![opts.disk, disk_show, &opts, (bold, bold)],
        item![opts.huge_pages, huge_pages_show, &opts, (bold, bold)],
        item![opts.load, load_show, &opts, (bold, bold)],
        item![opts.ssh, ssh_show, &opts, (bold, def)],
        item![opts.netif, netif_show, &opts, (bold.dimmed(), def.dimmed())],
        item![opts.netns, net_namespace, &opts, (bold, bold)],
        item![opts.k8s, k8s_show, &opts, (color, color.dimmed())],
        item![opts.nix, nix_show, &opts, (color, color.dimmed())],
        item![opts.venv, venv_show, &opts, (color, color.dimmed())],
        item![opts.manifest, manifest_show, &opts, (color, color.dimmed())],
    ];

    let vcs_fut = async {
        let vcs = if opts.vcs {
            env::current_dir()
                .ok()
                .and_then(|cwd| infer_vcs(cwd, &opts))
        } else {
            None
        };
        let vcs_ref = vcs.as_ref();
        let vcs_tasks = hlist![
            item_vcs![
                vcs_ref,
                <Vcs as VcsTrait>::branch,
                &opts,
                (bold, color.bold())
            ],
            item_vcs![
                vcs_ref,
                <Vcs as VcsTrait>::action,
                &opts,
                (bold, color.italic())
            ],
            item_vcs![vcs_ref, <Vcs as VcsTrait>::status, &opts, (bold, color)],
            item_vcs![vcs_ref, <Vcs as VcsTrait>::stash, &opts, (bold, def)],
            item_vcs![
                vcs_ref,
                <Vcs as VcsTrait>::worktree,
                &opts,
                (bold, bold.dimmed())
            ],
            item_vcs![vcs_ref, <Vcs as VcsTrait>::commit, &opts, (bold, bold)],
            item_vcs![vcs_ref, <Vcs as VcsTrait>::divergence, &opts, (bold, def)],
        ];
        vcs_tasks.hjoin().await
    };

    let post_vcs = hlist![
        item![opts.jobs, jobs_show, &opts, (color, color.bold())],
        item![opts.duration, duration_show, &opts, (def, def.dimmed())],
        item![
            opts.exit_code.is_some(),
            exit_code_show,
            &opts,
            (bold.red(), bold)
        ],
    ];

    // Poll all enabled providers concurrently on the single-threaded runtime,
    // without spawning heap tasks or cloning Arc/PathBuf.
    let (pre, vcs_res, post) = tokio::join!(pre_vcs.hjoin(), vcs_fut, post_vcs.hjoin());

    if let Some(start) = start {
        pre.print_timings();
        vcs_res.print_timings();
        post.print_timings();
        println!("{:<40} -> {:>15?}", "total time", start.elapsed());
    } else {
        let mut out = String::with_capacity(256);
        pre.write_chunks(&mut out);
        vcs_res.write_chunks(&mut out);
        post.write_chunks(&mut out);
        if !out.is_empty() {
            let _ = std::io::stdout().write_all(out.as_bytes());
        }
    }

    Ok(())
}

trait HListJoin {
    type Output;
    async fn hjoin(self) -> Self::Output;
}

impl HListJoin for HNil {
    type Output = HNil;
    #[inline]
    async fn hjoin(self) -> Self::Output {
        HNil
    }
}

impl<H, T> HListJoin for HCons<H, T>
where
    H: Future,
    T: HListJoin,
{
    type Output = HCons<H::Output, T::Output>;
    #[inline]
    async fn hjoin(self) -> Self::Output {
        let (head, tail) = tokio::join!(self.head, self.tail.hjoin());
        HCons { head, tail }
    }
}

trait HListRender {
    fn write_chunks(self, out: &mut String);
    fn print_timings(self);
}

impl HListRender for HNil {
    #[inline]
    fn write_chunks(self, _out: &mut String) {}
    #[inline]
    fn print_timings(self) {}
}

impl<C, T> HListRender for HCons<(&'static str, Duration, Option<Chunk<C>>), T>
where
    C: Display,
    T: HListRender,
{
    #[inline]
    fn write_chunks(self, out: &mut String) {
        if let (_, _, Some(c)) = self.head {
            let _ = write!(out, "{c} ");
        }
        self.tail.write_chunks(out);
    }

    fn print_timings(self) {
        let (f, dur, c) = self.head;
        let f = f.replace("auraline::providers::", "");
        if let Some(chunk) = c {
            println!("{f:<40} -> {dur:>15?} : ({chunk})");
        } else {
            println!("{f:<40} -> {dur:>15?} : (_)");
        }
        self.tail.print_timings();
    }
}

#[inline]
fn provider_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}
