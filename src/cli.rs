use clap::ValueHint;
use std::path::PathBuf;

static LONG_ABT: &str = r#"
crawn - A utility for fetching text from webpages with BFS expanding from a single page's URL

• crawn provides a way to fetch text from webpages from a single URL, unlike traditional tools, crawn expands into other pages and fetches them by extracting URLs from pages an fetching them.
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
    #[arg(required = true, value_hint = ValueHint::Url)]
    pub url: String,

    #[arg(short, long, required = true, value_hint = ValueHint::FilePath)]
    pub output: PathBuf,

    #[arg(short, long, value_hint = ValueHint::FilePath, global = true)]
    pub log_file: Option<PathBuf>,

    #[arg(long, global = true, conflicts_with = "include_text")]
    pub include_content: bool,

    #[arg(long, global = true, conflicts_with = "include_content")]
    pub include_text: bool,

    #[arg(short, long, global = true)]
    pub max_depth: Option<u8>,

    #[arg(short, long, global = true)]
    pub verbose: bool,
}
