use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long, help = "Specify a path where to run the line ($PWD by default)")]
    pub path: Option<String>,

    #[clap(short, long, help = "Specify the theme color")]
    pub theme: Option<String>,

    #[clap(short, long, help = "Enable short mode")]
    pub short_mode: bool,

    #[clap(short, long, help = "Fast mode")]
    pub fast: bool,

    #[clap(short, long, help = "Use Nerd Fonts")]
    pub nerd_font: bool,
}
