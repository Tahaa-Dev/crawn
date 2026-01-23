use std::sync::LazyLock;

use clap::Parser;
use owo_colors::OwoColorize;

mod cli;
mod crawler;
mod error;
mod fetch;
mod output;
mod repo;

use crawler::*;
pub(crate) use repo::*;

#[doc(hidden)]
pub(crate) static ARGS: LazyLock<cli::Args> = LazyLock::new(cli::Args::parse);

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match crawn().await {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {}", "FATAL:".red().bold(), e);
            std::process::ExitCode::FAILURE
        }
    }
}
