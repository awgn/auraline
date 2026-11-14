use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use frunk::Semigroup;
use smol_str::SmolStr;

#[derive(ValueEnum, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    #[default]
    None,
    Bash,
    Zsh,
}

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None, disable_help_flag = true, disable_help_subcommand = true)]
pub struct Cli {
    #[arg(long, global = true, action = ArgAction::Help, help = "Print help information")]
    pub help: Option<bool>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Prompt(Options),
    Init(InitOptions),
}

impl Default for Commands {
    fn default() -> Self {
        Self::Prompt(Default::default())
    }
}

#[derive(Args, Debug, Default)]
pub struct InitOptions {
    pub shell: SmolStr,
}

#[derive(Args, Debug, Default, Clone)]
pub struct Options {
    #[clap(short('u'), long, help = "Basic user info")]
    pub user: bool,

    #[clap(short('r'), long, help = "Basic realname info")]
    pub realname: bool,

    #[clap(short('h'), long, help = "Basic hostname info")]
    pub hostname: bool,

    #[clap(short('d'), long, help = "Basic devicename info")]
    pub device_name: bool,

    #[clap(short('D'), long, help = "Basic distro info")]
    pub distro: bool,

    #[clap(short('w'), long, help = "Current working directory")]
    pub pwd: bool,

    #[clap(short('W'), long, help = "Current working directory (full path)")]
    pub full_pwd: bool,

    #[clap(short('L'), long, help = "Show nested subshell depth")]
    pub shlvl: bool,

    #[clap(short('v'), long, help = "Show VCS info (git, hg, jj, etc.)")]
    pub vcs: bool,

    #[clap(short('s'), long, help = "Show SSH info")]
    pub ssh: bool,

    #[clap(short('S'), long, help = "Show cached sudo credentials status")]
    pub sudo: bool,

    #[clap(short('o'), long, help = "Show OS info")]
    pub os: bool,

    #[clap(short('V'), long, help = "Show virtualization / container info")]
    pub virt: bool,

    #[clap(
        short('p'),
        long,
        help = "Show Python / Conda virtual environment info"
    )]
    pub venv: bool,

    #[clap(short('x'), long, help = "Show Nix shell / direnv environment info")]
    pub nix: bool,

    #[clap(short('n'), long, help = "Show network interfaces")]
    pub netif: bool,

    #[clap(short('N'), long, help = "Show network namespace info")]
    pub netns: bool,

    #[clap(short('K'), long, help = "Show Kubernetes context and namespace")]
    pub k8s: bool,

    #[clap(short('m'), long, help = "Show memory usage info")]
    pub memory: bool,

    #[clap(short('k'), long, help = "Show disk usage info")]
    pub disk: bool,

    #[clap(short('l'), long, help = "Show system load average info")]
    pub load: bool,

    #[clap(
        short('P'),
        long,
        help = "Show Linux PSI contention alerts (CPU, memory, I/O)"
    )]
    pub psi: bool,

    #[clap(short('g'), long, help = "Show Linux CPU scaling governor")]
    pub cpu_gov: bool,

    #[clap(short('H'), long, help = "Show HugePages info")]
    pub huge_pages: bool,

    #[clap(
        short('M'),
        long,
        help = "Show development package info in the current directory"
    )]
    pub manifest: bool,

    #[clap(short('j'), long, help = "Show background / suspended shell jobs")]
    pub jobs: bool,

    #[clap(
        long,
        hide = true,
        help = "Number of background / suspended shell jobs (passed by shell hook)"
    )]
    pub jobs_count: Option<u32>,

    #[clap(short('e'), long, help = "Show the duration of the last command")]
    pub duration: bool,

    #[clap(
        long,
        hide = true,
        help = "Duration of the last command in nanoseconds (passed by shell hook)"
    )]
    pub duration_ns: Option<u64>,

    #[clap(long, help = "Specify the exit-code of the last command to show")]
    pub exit_code: Option<u8>,

    #[clap(
        long,
        value_enum,
        help = "Wrap non-printing ANSI sequences for shell prompt"
    )]
    pub shell: Option<ShellMode>,

    #[clap(long, help = "Enable timings mode (dev)")]
    pub timings: bool,

    #[clap(long, help = "Specify the theme color")]
    pub theme: Option<SmolStr>,

    #[clap(long, help = "Use Nerd Fonts")]
    pub nerd_font: bool,

    #[clap(long, help = "Specify the prompt profile to use: minimal, lean, nerdy")]
    pub profile: Option<SmolStr>,
}

impl Options {
    pub fn select_str<'a>(&self, normal: &'a str, nerd: &'a str) -> &'a str {
        if self.nerd_font {
            nerd
        } else {
            normal
        }
    }

    pub fn shell_mode(&self) -> ShellMode {
        if self.timings {
            ShellMode::None
        } else {
            self.shell.unwrap_or(ShellMode::None)
        }
    }

    pub fn has_explicit_provider(&self) -> bool {
        self.user
            || self.realname
            || self.hostname
            || self.device_name
            || self.distro
            || self.pwd
            || self.full_pwd
            || self.shlvl
            || self.vcs
            || self.ssh
            || self.sudo
            || self.os
            || self.virt
            || self.venv
            || self.nix
            || self.netif
            || self.netns
            || self.k8s
            || self.memory
            || self.disk
            || self.load
            || self.psi
            || self.cpu_gov
            || self.huge_pages
            || self.manifest
            || self.jobs
            || self.duration
    }
}

impl Semigroup for Options {
    fn combine(&self, other: &Self) -> Self {
        Self {
            user: self.user || other.user,
            realname: self.realname || other.realname,
            hostname: self.hostname || other.hostname,
            device_name: self.device_name || other.device_name,
            distro: self.distro || other.distro,
            pwd: self.pwd || other.pwd,
            full_pwd: self.full_pwd || other.full_pwd,
            shlvl: self.shlvl || other.shlvl,
            vcs: self.vcs || other.vcs,
            ssh: self.ssh || other.ssh,
            sudo: self.sudo || other.sudo,
            os: self.os || other.os,
            virt: self.virt || other.virt,
            venv: self.venv || other.venv,
            nix: self.nix || other.nix,
            netif: self.netif || other.netif,
            netns: self.netns || other.netns,
            k8s: self.k8s || other.k8s,
            memory: self.memory || other.memory,
            disk: self.disk || other.disk,
            load: self.load || other.load,
            psi: self.psi || other.psi,
            cpu_gov: self.cpu_gov || other.cpu_gov,
            huge_pages: self.huge_pages || other.huge_pages,
            manifest: self.manifest || other.manifest,
            jobs: self.jobs || other.jobs,
            jobs_count: self.jobs_count.or(other.jobs_count),
            duration: self.duration || other.duration,
            duration_ns: self.duration_ns.or(other.duration_ns),
            exit_code: self.exit_code.or(other.exit_code),
            shell: self.shell.or(other.shell),
            timings: self.timings || other.timings,
            theme: self.theme.clone().or(other.theme.clone()),
            nerd_font: self.nerd_font || other.nerd_font,
            profile: self.profile.clone().or(other.profile.clone()),
        }
    }
}
