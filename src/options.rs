use clap::{ArgAction, Args, Parser, Subcommand};
use frunk::Semigroup;
use smol_str::SmolStr;

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

#[derive(Args, Debug, Default)]
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

    #[clap(short('v'), long, help = "Show VCS info (git, hg, jj, etc.)")]
    pub vcs: bool,

    #[clap(short('s'), long, help = "Show SSH info")]
    pub ssh: bool,

    #[clap(short('o'), long, help = "Show OS info")]
    pub os: bool,

    #[clap(short('V'), long, help = "Show virtual env info")]
    pub virt: bool,

    #[clap(short('n'), long, help = "Show network interfaces")]
    pub netif: bool,

    #[clap(short('N'), long, help = "Show network namespace info")]
    pub netns: bool,

    #[clap(short('m'), long, help = "Show memory usage info")]
    pub memory: bool,

    #[clap(short('H'), long, help = "Show HugePages info")]
    pub huge_pages: bool,

    #[clap(
        short('M'),
        long,
        help = "Show development package info in the current directory"
    )]
    pub manifest: bool,

    #[clap(short('e'), long, help = "Show the duration of the last command)")]
    pub duration: bool,

    #[clap(long, help = "Specify the exit-code of the last command to show")]
    pub exit_code: Option<u8>,

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
            vcs: self.vcs || other.vcs,
            ssh: self.ssh || other.ssh,
            os: self.os || other.os,
            virt: self.virt || other.virt,
            netif: self.netif || other.netif,
            netns: self.netns || other.netns,
            memory: self.memory || other.memory,
            huge_pages: self.huge_pages || other.huge_pages,
            manifest: self.manifest || other.manifest,
            duration: self.duration || other.duration,
            exit_code: self.exit_code.or(other.exit_code),
            timings: self.timings || other.timings,
            theme: self.theme.clone().or(other.theme.clone()),
            nerd_font: self.nerd_font || other.nerd_font,
            profile: self.profile.clone().or(other.profile.clone()),
        }
    }
}
