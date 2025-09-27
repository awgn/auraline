use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long, help = "Specify the theme color")]
    pub theme: Option<String>,

    #[clap(short, long, help = "Enable short mode")]
    pub short_mode: bool,

    #[clap(short, long, help = "Use Nerd Fonts")]
    pub nerd_font: bool,

    #[clap(long, help = "Specify the exit-code of the last command")]
    pub exit_code: Option<u8>,

    #[clap(short('H'), long, help = "Basic hostname info")]
    pub hostname: bool,

    #[clap(short, long, help = "Basic devicename info")]
    pub device_name: bool,

    #[clap(short, long, help = "Basic user info")]
    pub user: bool,

    #[clap(short, long, help = "Basic realname info")]
    pub realname: bool,

    #[clap(short('D'), long, help = "Basic distro info")]
    pub distro: bool,

    #[clap(short('w'), long, help = "Current working directory")]
    pub pwd: bool,

    #[clap(short('W'), long, help = "Current working directory")]
    pub full_pwd: bool,

    #[clap(long, help = "Enable timings mode (dev)")]
    pub timings: bool,
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
