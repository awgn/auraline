#![recursion_limit = "256"]
mod chunk;
mod cmd;
mod commands;
mod options;
mod providers;
mod style;
mod sys;

use crate::options::{Cli, Options};

use anyhow::Context;
use clap::Parser;
use frunk::Semigroup;
use smol_str::ToSmolStr;

fn resolve_prompt_options(
    cli_opts: Options,
    env_profile: Option<&str>,
    env_options_str: Option<&str>,
    env_theme: Option<&str>,
) -> anyhow::Result<Options> {
    let mut options = cli_opts;

    if let Some(opts_str) = env_options_str.map(str::trim).filter(|s| !s.is_empty()) {
        let mut args = vec!["auraline", "prompt"];
        args.extend(opts_str.split_whitespace());
        let env_cli = Cli::try_parse_from(args).with_context(|| "AURALINE_OPTIONS parse error")?;

        if let options::Commands::Prompt(env_opts) = env_cli.command {
            options = options.combine(&env_opts);
        }
    }

    let profile_name = options.profile.clone().or_else(|| {
        env_profile
            .filter(|s| !s.is_empty())
            .map(ToSmolStr::to_smolstr)
    });

    if let Some(name) = profile_name {
        let profile_opts = commands::profile::get_profile(&name)
            .with_context(|| format!("profile '{name}' not found"))?;
        options = options.combine(&profile_opts);
    } else if !options.has_explicit_provider() {
        options = options.combine(&commands::profile::MINIMAL);
    }

    if let Some(theme) = env_theme.filter(|s| !s.is_empty()) {
        options.theme = options.theme.or_else(|| Some(theme.to_smolstr()));
    }

    Ok(options)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        options::Commands::Init(options) => {
            commands::init::print_init(options);
        }

        options::Commands::Prompt(cli_opts) => {
            let env_profile = std::env::var("AURALINE_PROFILE").ok();
            let env_options = std::env::var("AURALINE_OPTIONS").ok();
            let env_theme = std::env::var("AURALINE_THEME").ok();

            let options = resolve_prompt_options(
                cli_opts,
                env_profile.as_deref(),
                env_options.as_deref(),
                env_theme.as_deref(),
            )?;

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("failed to initialize async runtime")?;

            rt.block_on(commands::prompt::print_prompt(options))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_prompt_options;
    use crate::options::Options;
    use smol_str::SmolStr;

    #[test]
    fn test_cli_profile_overrides_env_profile() {
        let cli = Options {
            profile: Some(SmolStr::new_static("minimal")),
            ..Default::default()
        };
        let resolved = resolve_prompt_options(cli, Some("nerdy"), None, None).unwrap();
        assert!(resolved.user);
        assert!(resolved.pwd);
        assert!(!resolved.vcs);
        assert!(!resolved.nerd_font);
    }

    #[test]
    fn test_default_minimal_when_no_profile_and_no_flags() {
        let resolved = resolve_prompt_options(Options::default(), None, None, None).unwrap();
        assert!(resolved.user);
        assert!(resolved.hostname);
        assert!(resolved.pwd);
        assert!(!resolved.vcs);
    }

    #[test]
    fn test_explicit_provider_without_profile_does_not_enable_minimal() {
        let cli = Options {
            manifest: true,
            ..Default::default()
        };
        let resolved = resolve_prompt_options(cli, None, None, None).unwrap();
        assert!(resolved.manifest);
        assert!(!resolved.user);
        assert!(!resolved.pwd);
    }

    #[test]
    fn test_invalid_profile_returns_error() {
        let cli = Options {
            profile: Some(SmolStr::new_static("unknown_prof")),
            ..Default::default()
        };
        assert!(resolve_prompt_options(cli, None, None, None).is_err());
        assert!(resolve_prompt_options(Options::default(), Some("typo"), None, None).is_err());
    }
}
