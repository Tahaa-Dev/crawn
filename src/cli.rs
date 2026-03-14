use clap::ValueHint;
use std::path::PathBuf;

static LONG_ABT: &str = r#"
crawn - A utility for web crawling and scraping

  -> crawn provides a simple way for crawling URLs and scraping HTML from them.
  -> crawn has exceptional speed due to being built in optimized Rust with tokio async.
  -> Easy to debug errors with detailed logs.

╭──────────────────────·Examples·──────────────────────╮
│                                                  ••• │
│ crawn https://ex.com/index.html > out.ndjson         │
│ crawn https://ex.com/index.html -l f.log | jq -s '.' │
│ crawn https://ex.com/index.html -v | grep 'captcha'  │
│                                                      │
╰──────────────────────────────────────────────────────╯
"#;

#[derive(clap::Parser)]
#[command(
    author,
    version,
    about = "A utility for web crawling and scraping",
    long_about = LONG_ABT
)]
pub struct Args {
    /// The starting URL to crawl
    /// If it is not providied as an arg, it is extracted from Stdin
    #[arg(value_hint = ValueHint::Url)]
    pub url: Option<String>,

    /// Optional log file path (logs to stdout if not provided)
    #[arg(short, long, value_hint = ValueHint::FilePath, global = true)]
    pub log_file: Option<PathBuf>,

    /// Include full HTML content in output (mutually exclusive with --include-text)
    #[arg(long, global = true, conflicts_with = "include_text")]
    pub include_content: bool,

    /// Include extracted text in output (mutually exclusive with --include-content)
    #[arg(long, global = true, conflicts_with = "include_content")]
    pub include_text: bool,

    /// Maximum crawl depth (default: 4)
    #[arg(short, long, global = true)]
    pub max_depth: Option<u8>,

    /// Enable verbose logging - logs all HTTP requests instead of error warnings only
    #[arg(short, long, global = true)]
    pub verbose: bool,
}
