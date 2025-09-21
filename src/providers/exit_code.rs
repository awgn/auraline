use crate::{options::Options, style::to_superscript};

pub async fn show(opts: &Options) -> Option<String> {
    let exit_code = opts.exit_code?;
    if exit_code == 0 {
        None
    } else {
        let mut buffer = itoa::Buffer::new();
        let exit_code = buffer.format(exit_code);
        Some(format!("✗{}", to_superscript(exit_code)))
    }
}
