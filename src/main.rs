#![recursion_limit = "256"]
mod chunk;
mod cmd;
mod commands;
mod presets;
mod prompt;
mod providers;
mod style;

use crate::{
    commands::{Cli, Options},
    presets::get_preset,
};

use anyhow::Context;
use clap::Parser;
use frunk::Semigroup;
use prompt::print_prompt;
use smol_str::ToSmolStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        commands::Commands::Init(_options) => todo!(),

        commands::Commands::Prompt(mut options) => {
            // combine preset options (either specified by command line or env variable)
            let preset_name = options.preset.clone().or_else(|| {
                std::env::var("AURALINE_PRESET")
                    .ok()
                    .map(|s| s.to_smolstr())
            });

            if let Some(preset_name) = preset_name {
                let preset_name = preset_name.to_smolstr();
                let preset_opts = get_preset(&preset_name)
                    .with_context(|| format!("preset '{preset_name}' not found"))?;
                options = options.combine(&preset_opts);
            }

            // Combine with additional options from the AURALINE_OPTIONS environment variable.
            if let Ok(opts_str) = std::env::var("AURALINE_OPTIONS") {
                let mut args = vec!["auraline", "prompt"];
                args.extend(opts_str.split_whitespace());
                let env_cli = Cli::try_parse_from(args)
                    .with_context(|| "AURALINE_OPTIONS parse error")?;

                if let commands::Commands::Prompt(env_opts) = env_cli.command {
                    options = options.combine(&env_opts);
                }
            }

            print_prompt(options).await?;
        }
    }

    Ok(())
}
