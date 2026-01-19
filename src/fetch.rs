use owo_colors::OwoColorize;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    UrlRepo,
    error::{Res, ResExt},
};

pub(crate) async fn fetch_url(url: &str, client: &reqwest::Client) -> Res<String> {
    // Reuse reqwest::Client for performance
    let res = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch URL: {}", url.bright_blue().italic()))?;

    let text = res.text().await.with_context(|| {
        format!(
            "Failed to fetch HTML (text) from URL: {}",
            url.bright_blue().italic()
        )
    })?;

    Ok(text)
}

pub(crate) fn extract_links<R: UrlRepo>(
    document: &Html,
    repo: &mut R,
    base: &Url,
    anchor_selector: &Selector,
) -> Res<()> {
    for url in document.select(anchor_selector) {
        if let Some(href) = url.attr("href") {
            let abs = base
                .join(href.trim_end_matches('/'))
                .with_context(|| format!("Failed to resolve relative URL: {}", href))?;

            repo.add(normalize_url(abs)?);
        } else {
            unreachable!()
        }
    }

    Ok(())
}

pub(crate) fn extract_keywords(document: &Html, meta_selector: &Selector) -> String {
    let mut res = String::new();

    if let Some(meta) = document.select(meta_selector).next() {
        if let Some(keywords) = meta.attr("content") {
            res.push_str(keywords);
        }
    }

    // normalize whitespaces
    res.replace(" ", "").to_lowercase()
}

pub(crate) fn extract_text(document: &Html, body_selector: &Selector) -> String {
    if let Some(body) = document.select(&body_selector).next() {
        body.text().collect()
    } else {
        String::new()
    }
}

pub(crate) fn extract_title(document: &Html, title_selector: &Selector) -> String {
    if let Some(title) = document.select(title_selector).next() {
        title.text().collect()
    } else {
        String::new()
    }
}

fn normalize_url(mut url: Url) -> Res<String> {
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap_or(());
    }

    if let Some(domain) = url.domain() {
        let res = url.set_host(Some(&domain.to_lowercase()));
        res.context("Failed to set host domain for URL")?;
    }

    url.set_fragment(None);

    Ok(url.to_string())
}
