use std::path::PathBuf;

static LONG_ABT: &str = r#"
crawn - A utility for fetching text from webpages with BFS expanding from a single page's URL

• crawn provides a way to fetch text from webpages from a single URL, unlike traditional tools, crawn expands into other pages and fetches them by extracting URLs from pages and fetching them.
• crawn has exceptional speed due to being built in optimized Rust with tokio async.
"#;

#[derive(clap::Parser)]
#[command(
    author,
    version,
    about = "A utility for fetching text from webpages with BFS expanding from a single page's URL",
    long_about = LONG_ABT
)]
pub struct Args {
    /// Argument for setting the base URL for fetching
    #[arg(required = true, value_hint = clap::ValueHint::Url)]
    pub url: String,

    /// Argument for setting file to export output to
    /// Will abort if file doesn's have ".ndjson" extension
    #[arg(required = true, value_hint = clap::ValueHint::FilePath, global = true)]
    pub output: PathBuf,

    #[arg(short, long, value_hint = clap::ValueHint::FilePath, global = true)]
    pub log_file: Option<PathBuf>,
}
