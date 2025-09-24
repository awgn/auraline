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

    #[clap(long, help = "Enable timing mode (dev)")]
    pub timing: bool,
}
