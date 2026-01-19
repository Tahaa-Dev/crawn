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
use error::*;
pub(crate) use fetch::*;
pub(crate) use repo::*;

#[doc(hidden)]
pub(crate) static ARGS: LazyLock<cli::Args> = LazyLock::new(cli::Args::parse);

async fn run() -> Res<()> {
    let res = "TEMP".red();
    println!("Fetched: {:?}", res);

    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "FATAL:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
