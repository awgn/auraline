#![recursion_limit = "256"]
mod chunk;
mod cmd;
mod commands;
mod options;
mod providers;
mod style;

use crate::options::{Cli, Options};

use anyhow::Context;
use clap::Parser;
use frunk::Semigroup;
use smol_str::ToSmolStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        options::Commands::Init(options) => {
            commands::init::print_init(options);
        }

        options::Commands::Prompt(mut options) => {
            // combine profile options (either specified by command line or env variable)
            let profile_name = options.profile.clone().or_else(|| {
                std::env::var("AURALINE_PROFILE")
                    .ok()
                    .map(|s| s.to_smolstr())
            });

            if let Some(profile_name) = profile_name {
                let profile_name = profile_name.to_smolstr();
                let profile_opts = commands::profile::get_profile(&profile_name)
                    .with_context(|| format!("profile '{profile_name}' not found"))?;
                options = options.combine(&profile_opts);
            }

            // Combine with additional options from the AURALINE_OPTIONS environment variable.
            if let Ok(opts_str) = std::env::var("AURALINE_OPTIONS") {
                let mut args = vec!["auraline", "prompt"];
                args.extend(opts_str.split_whitespace());
                let env_cli =
                    Cli::try_parse_from(args).with_context(|| "AURALINE_OPTIONS parse error")?;

                if let options::Commands::Prompt(env_opts) = env_cli.command {
                    options = options.combine(&env_opts);
                }
            }

            commands::prompt::print_prompt(options).await?;
        }
    }

    Ok(())
}
