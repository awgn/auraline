use crate::options::InitOptions;
use phf::phf_map;

static INIT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "bash" => include_str!("scripts/init.bash"),
    "zsh" => include_str!("scripts/init.zsh"),
};

pub fn print_init(opts: InitOptions) {
    match INIT_MAP.get(opts.shell.as_str()) {
        Some(&script) => print!("{script}"),
        None => {
            eprintln!("Unsupported shell: '{}'", opts.shell);
            eprintln!("Supported shells are:");
            for key in INIT_MAP.keys() {
                eprintln!(" - {}", key);
            }
            std::process::exit(1);
        }
    };
}
