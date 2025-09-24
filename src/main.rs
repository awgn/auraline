mod chunk;
mod cmd;
mod options;
mod prompt;
mod providers;
mod style;

use crate::options::Options;
use clap::Parser;
use prompt::print_prompt;

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    print_prompt(Options::parse()).await
}
