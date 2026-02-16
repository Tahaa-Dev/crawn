//! **A utility for web crawling and scraping**
//!
//! ## Usage
//!
//! - Basic Crawling:
//! ```bash
//! crawn -o output.ndjson https://example.com
//! ```
//!
//! - With Logging:
//! ```bash
//! crawn -o output.ndjson -l crawler.log https://example.com
//! ```
//!
//! - Verbose Mode (Log All Requests):
//! ```bash
//! crawn -o output.ndjson -v https://example.com
//! ```
//!
//! - Custom Depth Limit:
//! ```bash
//! crawn -o output.ndjson -m 3 https://example.com
//! ```
//!
//! - Full HTML:
//! ```bash
//! crawn -o output.ndjson --include-content https://example.com
//! ```
//!
//! - Extracted text only:
//! ```bash
//! crawn -o output.ndjson --include-text https://example.com
//! ```
//!
//! ---
//!
//! ## Output Format
//!
//! Results are written as NDJSON (newline-delimited JSON):
//! ```json
//! {"URL": "https://example.com", "Title": "Example Domain", "Links": 12}
//! {"URL": "https://example.com/about", "Title": "About Us", "Links": 9}
//! {"URL": "https://example.com/contact", "Title": "Contact", "Links": 48}
//! ```
//!
//! - With `--include-text`:
//! ```json
//! {"URL": "https://example.com", "Title": "Example Domain", "Links": 27, "Text": "Example Domain\nThis domain is..."}
//! ```
//!
//! - With `--include-content`:
//! ```json
//! {"URL": "https://example.com", "Title": "Example Domain", "Links": 30, "Content": "<!DOCTYPE html>\n<html>..."}
//! ```
//!
//! ---
//!
//! ## How It Works
//!
//! 1. BFS Crawling:
//! - Starts at the seed URL (depth 0)
//! - Discovers links on each page
//! - Processes links level-by-level (breadth-first)
//! - Stops at max_depth (default: 3)
//! - Uses `tokio::task::spawn` and `tokio::task::spawn_blocking` for concurrent processing
//!
//! 2. Keyword Filtering:
//! - Extracts "keywords" from URL paths (sanitized, lowercased)
//! - Splits by /, -, _ (e.g., /rust-tutorials/async → ["rust", "tutorials", "async"])
//! - Filters stop words, numbers, short words (<3 chars)
//! - Matches candidate URLs against base keywords
//! - Result: Only crawls relevant pages, skips off-topic content
//!
//! 3. Rate Limiting:
//! - Random range between 300 - 600ms
//! - Prevents server overload and IP bans
//! - Configurable via code (not exposed as CLI flag yet)
//!
//! 4. Error Handling:
//! - Network errors: Logged as warnings, crawling continues
//! - HTTP 404/500: Skipped, logged as warnings
//! - Parse failures: Logged, returns empty JSON
//! - Fatal errors: Printed to stdout with full context chain
//!
//! ---
//!
//! ## Logging
//!
//! #### Log Levels:
//!
//! - **INFO** (verbose mode only): Request logs
//! - **WARN** (always): Recoverable errors (404, network timeouts)
//! - **FATAL** (always): Unrecoverable errors (invalid URL, disk full)
//!
//! #### Log Format:
//!
//! ```text
//! 2026-01-24 02:37:40.351 [INFO]:
//! Sent request to URL: https://example.com
//!
//! 2026-01-24 02:37:41.123 [WARN]:
//! Failed to fetch URL: https://example.com/broken-link
//! Cause: HTTP 404 Not Found
//! ```
//!
//! ---
//!
//! ## Examples
//!
//! - Crawl Documentation Site:
//! ```bash
//! crawn -o rust-docs.ndjson https://doc.rust-lang.org/book/
//! ```
//!
//! - Crawl with Logging:
//! ```bash
//! crawn -o output.ndjson -l crawler.log -v https://example.com
//! ```
//!
//! - Limit to 2 Levels Deep:
//! ```bash
//! crawn -o shallow.ndjson -m 2 https://example.com
//! ```
//!
//! ---
//!
//! ## Limitations
//!
//! - Same-domain only (no external links, by design)
//! - No JavaScript rendering (static HTML only)
//! - No authentication (public pages only)
//!
//! ---
//!
//! ## License
//!
//! crawn is licensed under the **MIT** license.

use std::sync::atomic::{AtomicU8, AtomicUsize};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use clap::Parser;
use owo_colors::OwoColorize;
use tokio::sync::Mutex;

mod cli;
mod crawler;
mod error;
mod fetch;
mod output;
mod repo;

use crate::fetch::*;
use crawler::*;
pub use repo::*;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{LOG_TIMESTAMP_FORMAT, Log, Res, ResExt};
use crate::output::{flush_writer, write_output};

pub static ARGS: LazyLock<cli::Args> = LazyLock::new(cli::Args::parse);
static CRAWLED: LazyLock<Arc<AtomicUsize>> = LazyLock::new(|| Arc::new(AtomicUsize::new(0)));
static SUCCESSES: LazyLock<Arc<AtomicUsize>> = LazyLock::new(|| Arc::new(AtomicUsize::new(0)));

async fn run() -> Res<()> {
    let args = &*ARGS;
    let repo = Arc::new(Mutex::new(InMemoryRepo::default()));
    let client = Arc::new(CrawnClient::new()?);
    let curr_depth = Arc::new(AtomicU8::new(0));
    let pending = Arc::new(AtomicUsize::new(0));
    let crawled = Arc::clone(&*CRAWLED);
    let successes = Arc::clone(&*SUCCESSES);
    let url = &args.url;
    let base = Url::parse(url).context("Failed to parse base URL")?;

    let base_keywords = Arc::new(get_keywords(&base));

    let base_domain = Arc::new(base.domain().unwrap_or_default().to_owned());

    let selectors = Arc::new(Selectors {
        anchor: Selector::parse("a[href]").with_context(format_args!(
            "Failed to parse selector for HTML 'anchor' (link) tag: {}",
            "`<a href=\"URL\">`".yellow()
        ))?,

        title: Selector::parse("title").with_context(format_args!(
            "Failed to parse selector for HTML 'title' tag: {}",
            "`<title>`".yellow()
        ))?,

        body: if args.include_text {
            Some(Selector::parse("body").with_context(format_args!(
                "Failed to parse selector for HTML 'body' tag: {}",
                "`<body>`".yellow()
            ))?)
        } else {
            None
        },
    });

    crawled.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let content = fetch_url(url, Arc::clone(&client))
        .await
        .context("Failed to fetch base URL")?;

    successes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    if args.verbose {
        String::from("Fetched content from base URL")
            .log("[INFO]")
            .await?;
    }

    let doc = Html::parse_document(&content);

    let links = extract_links(&doc, Arc::new(base), &selectors.anchor);
    let mut link_count = 0usize;
    {
        let mut rp = repo.lock().await;
        rp.mark(url.clone())
            .await
            .context("Failed to mark base URL as visited")?;

        for link in links {
            let link = match_option!(link.log("[WARN]").await?);
            let link = match_option!(normalize_url(link).log("[WARN]").await?);

            match_option!(rp.add(link).await.log("[WARN]").await?);

            link_count += 1;
        }
        rp.add(String::from("M")).await?;
    }
    curr_depth.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let text = selectors
        .body
        .as_ref()
        .map(|body_selector| extract_text(&doc, body_selector));
    let title = extract_title(&doc, &selectors.title);

    let content = if args.include_content {
        Some(content)
    } else {
        None
    };

    write_output(url.clone(), title, link_count, text, content)
        .await
        .log("[WARN]")
        .await?;

    let task_count = if args.include_content || args.include_text {
        6
    } else {
        9
    };

    let mut tasks = Vec::new();
    for _ in 0..task_count {
        let repo = Arc::clone(&repo);
        let base_keywords = Arc::clone(&base_keywords);
        let base_domain = Arc::clone(&base_domain);
        let selectors = Arc::clone(&selectors);
        let client = Arc::clone(&client);
        let curr_depth = Arc::clone(&curr_depth);
        let pending = Arc::clone(&pending);
        let crawled = Arc::clone(&crawled);
        let successes = Arc::clone(&successes);

        let task: tokio::task::JoinHandle<Res<()>> = tokio::task::spawn(async move {
            loop {
                if curr_depth.load(std::sync::atomic::Ordering::SeqCst)
                    > args.max_depth.unwrap_or(4)
                {
                    break;
                }

                let work_item = {
                    let mut repo_guard = repo.lock().await;
                    repo_guard.pop().await.log("[WARN]").await?.unwrap_or(None)
                };

                match work_item {
                    None => {
                        if pending.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        } else {
                            break;
                        }
                    }

                    Some(url) => {
                        if &url == "M" {
                            if pending.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                                #[allow(clippy::unit_arg)]
                                repo.lock()
                                    .await
                                    .kick(url)
                                    .await
                                    .log("[WARN]")
                                    .await?
                                    .unwrap_or({
                                        tokio::time::sleep(Duration::from_millis(100)).await;
                                    });

                                tokio::time::sleep(Duration::from_millis(100)).await;
                            } else {
                                repo.lock()
                                    .await
                                    .add(url)
                                    .await
                                    .log("[WARN]")
                                    .await?
                                    .unwrap_or_default();

                                curr_depth.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        } else {
                            pending.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                            let can_extract = curr_depth.load(std::sync::atomic::Ordering::SeqCst)
                                < args.max_depth.unwrap_or(4);

                            let other = Url::parse(&url)
                                .with_context(format_args!("Failed to parse URL: {}", &url))?;

                            if should_crawl(
                                Arc::clone(&base_domain),
                                Arc::clone(&base_keywords),
                                &other,
                            ) {
                                crawled.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                                let is_success = worker(
                                    Arc::clone(&repo),
                                    Arc::clone(&selectors),
                                    Arc::clone(&client),
                                    url,
                                    can_extract,
                                )
                                .await
                                .log("[WARN]")
                                .await?
                                .is_some();

                                if is_success {
                                    successes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                }
                            }

                            pending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                }
            }

            Ok(())
        });

        tasks.push(task);
    }

    for task in tasks {
        task.await.context("Failed to spawn concurrent worker")??;
    }

    flush_writer().await
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(_) => {
            eprintln!(
                "\n{} Crawled {} URLs with {} successes and {} failures",
                "Finished!".bright_green().bold(),
                CRAWLED
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .bright_green()
                    .bold(),
                SUCCESSES
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .bright_green()
                    .bold(),
                {
                    CRAWLED.fetch_sub(
                        SUCCESSES.load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    let temp = CRAWLED
                        .load(std::sync::atomic::Ordering::Relaxed)
                        .to_string();
                    if &temp == "0" {
                        temp.bright_green().bold().to_string()
                    } else {
                        temp.red().bold().to_string()
                    }
                }
            );
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            let timestamp: String = time::OffsetDateTime::now_utc()
                .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
                .format(&LOG_TIMESTAMP_FORMAT)
                .map_err(|_| String::from("Format Failure"))
                .context("Failed to format timestamp for log")
                .unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    String::from("")
                });

            eprintln!("{} {}:\n {}", timestamp.yellow(), "[FATAL]".red().bold(), e);
            std::process::ExitCode::FAILURE
        }
    }
}
