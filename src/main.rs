#![recursion_limit = "256"]
mod chunk;
mod cmd;
mod commands;
mod prompt;
mod providers;
mod style;

use crate::commands::{Cli, Options};
use clap::Parser;
use prompt::print_prompt;

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let cli = Cli::parse();
    match cli.command {
        commands::Commands::Startup(options) => todo!(),
        commands::Commands::Prompt(options) => print_prompt(options).await
    }
}
