mod cmd;
mod providers;
mod options;
mod prompt;
mod color;

use std::sync::Arc;

use crate::options::Options;
use clap::Parser;
use prompt::{print_prompt};

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let opts = Arc::new(Options::parse());
    print_prompt(opts).await
}
