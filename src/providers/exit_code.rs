use crate::{chunk::Chunk, commands::Options, style::to_superscript};
use smol_str::{format_smolstr, SmolStr};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    let exit_code = opts.exit_code?;
    if exit_code == 0 {
        None
    } else {
        let mut buffer = itoa::Buffer::new();
        let exit_code = buffer.format(exit_code);
        Some(Chunk::new(
            "✘",
            format_smolstr!("{}", to_superscript(exit_code)),
        ))
    }
}
