use owo_colors::OwoColorize;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    InMemoryRepo, UrlRepo,
    error::{Log, Res, ResExt},
    fetch::*,
    match_option,
    output::write_output,
};

pub(crate) async fn crawn() -> Res<()> {
    let args = &*crate::ARGS;
    let max_depth = args.depth.unwrap_or(4);

    let mut depth = 0u8;

    let mut repo = InMemoryRepo::new().await;

    let base_url = Url::parse(&args.url).with_context(|| {
        format!(
            "Failed to parse base URL: {}",
            &args.url.bright_blue().italic()
        )
    })?;
    let base_keywords = get_keywords(&base_url);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build client")?;

    let base_content = fetch_url(&args.url, &client)
        .await
        .context("Failed to fetch base URL")?;

    let body_selector = Selector::parse("body").with_context(|| {
        format!(
            "Failed to parse selector for HTML body tag: {}",
            "`<body>`".yellow()
        )
    })?;

    let anchor_selector = Selector::parse("a[href]").with_context(|| {
        format!(
            "Failed to parse selector for HTML anchor (link) tag: {}",
            "`<a href=\"URL\">`".yellow()
        )
    })?;

    let title_selector = Selector::parse("title").with_context(|| {
        format!(
            "Failed to parse selector for HTML title tag: {}",
            "`<title>`".yellow()
        )
    })?;

    let base_document = Html::parse_document(&base_content);

    let base_links = extract_links(&base_document, &mut repo, &base_url, &anchor_selector)
        .await
        .context("Failed to extract URLs from base URL")?;

    let base_title = extract_title(&base_document, &title_selector);

    let base_text: Option<String> = if args.include_text {
        Some(extract_text(&base_document, &body_selector))
    } else {
        None
    };

    if args.include_content {
        write_output(
            &args.url,
            &base_title,
            base_links,
            base_text.as_deref(),
            Some(&base_content),
        )
        .await
        .with_context(|| format!("Failed to write output entry for base URL: {}", &args.url.bright_blue().italic()))
        .log_err()
        .await?;
    } else {
        write_output(
            &args.url,
            &base_title,
            base_links,
            base_text.as_deref(),
            None,
        )
        .await
        .with_context(|| format!("Failed to write output entry for base URL: {}", &args.url.bright_blue().italic()))
        .log_err()
        .await?;
    }

    repo.add(String::from("M")).await?;
    depth += 1;

    while let Some(Some(raw_url)) = repo.pop().await.log_err().await?
        && depth <= max_depth
    {
        if raw_url == "M" {
            depth += 1;
            match_option!(repo.add(String::from("M")).await.log_err().await?);
        } else {
            let url_opt = Url::parse(&raw_url)
                .with_context(|| {
                    format!("Failed to parse URL: {}", &raw_url.bright_blue().italic())
                })
                .log_err()
                .await?;

            let url = match_option!(url_opt);

            if should_crawl(&base_url, &base_keywords, &url) {
                let content = match_option!(
                    fetch_url(&raw_url, &client)
                        .await
                        .with_context(|| format!(
                            "Failed to fetch URL: {}",
                            &raw_url.bright_blue().italic()
                        ))
                        .log_err()
                        .await?
                );

                let document = Html::parse_document(&content);

                let links = match_option!(
                    extract_links(&document, &mut repo, &url, &anchor_selector)
                        .await
                        .with_context(|| format!(
                            "Failed to extract URLs from URL: {}",
                            &raw_url.bright_blue().italic()
                        ))
                        .log_err()
                        .await?
                );

                let title = extract_title(&document, &title_selector);

                let text: Option<String> = if args.include_text {
                    Some(extract_text(&document, &body_selector))
                } else {
                    None
                };

                if args.include_content {
                    match_option!(
                        write_output(&raw_url, &title, links, text.as_deref(), Some(&content))
                            .await
                            .with_context(|| format!("Failed to write output entry for URL: {}", &raw_url.bright_blue().italic()))
                            .log_err()
                            .await?
                    );
                } else {
                    match_option!(
                        write_output(&raw_url, &title, links, text.as_deref(), None)
                            .await
                            .with_context(|| format!("Failed to write output entry for URL: {}", &raw_url.bright_blue().italic()))
                            .log_err()
                            .await?
                    );
                }
            }
        }
    }

    Ok(())
}

const GENERICS: [&str; 3] = ["tutorial", "guide", "blog"];

fn should_crawl(base_url: &Url, base_keywords: &[String], other: &Url) -> bool {
    if let Some(domain) = base_url.domain() {
        if let Some(other_domain) = other.domain() {
            if other_domain != domain {
                return false;
            }
        } else {
            resext::panic_if!(
                true,
                || format!(
                    "FATAL: Base URL: {} does not contain a valid host domain",
                    base_url.as_str().bright_blue().italic()
                ),
                1
            );
        }
    }

    let other_keywords = get_keywords(other);

    let match_count = other_keywords
        .iter()
        .filter(|kw| base_keywords.contains(kw) || GENERICS.contains(&kw.as_str()))
        .count();

    if match_count >= 2 {
        return true;
    }

    true
}

// common stop words
const STOP_WORDS: [&str; 9] = [
    "how", "to", "the", "and", "for", "with", "from", "about", "by",
];

fn get_keywords(url: &Url) -> Vec<String> {
    let mut url = url.clone();

    url.set_query(None);
    url.set_fragment(None);

    let path = url.path().to_lowercase();

    path.split(['/', '-', '_'])
        .filter(|s| {
            !s.chars().all(|c| c.is_numeric())
                && !s.is_empty()
                && s.len() >= 3
                && !STOP_WORDS.contains(s)
        })
        .map(|s| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect())
        .collect()
}
