use crate::options::InitOptions;
use phf::phf_map;

static INIT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "bash" => include_str!("scripts/init.bash"),
    "zsh" => include_str!("scripts/init.zsh"),
};

fn shell_quote_bin() -> String {
    match std::env::current_exe()
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok())
    {
        Some(path) => format!("'{}'", path.replace('\'', "'\\''")),
        None => "auraline".to_string(),
    }
}

pub fn print_init(opts: InitOptions) {
    match INIT_MAP.get(opts.shell.as_str()) {
        Some(&script) => {
            let bin = shell_quote_bin();
            let rendered = script.replace("__AURALINE_BIN__", &bin);
            print!("{rendered}");
        }
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
