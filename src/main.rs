mod color;
mod git;
mod options;
mod prompt;

use crate::options::Options;
use clap::Parser;
use futures::TryFutureExt;
use prompt::{build_prompt, print_prompt};

#[tokio::main]
async fn main() {
    let _ = build_prompt(Options::parse())
        .and_then(|p| async move { print_prompt(p).await })
        .await;
}
