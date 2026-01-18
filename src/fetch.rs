use owo_colors::OwoColorize;
use scraper::Selector;
use url::Url;

use crate::{
    UrlRepo,
    error::{Res, ResExt},
};

pub(crate) async fn fetch_url(url: &str, client: reqwest::Client) -> Res<String> {
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

pub(crate) fn extract_links<R: UrlRepo>(text: &str, repo: &mut R, base: &str, selector: &Selector) -> Res<()> {
    let document = scraper::Html::parse_document(text);

    let base = Url::parse(base)
        .context("Failed to parse base URL")?;

    for url in document.select(selector) {
        if let Some(href) = url.attr("href") {
            let abs = base.join(href.trim_end_matches('/'))
                .with_context(|| format!("Failed to resolve relative URL: {}", href))?;

            repo.add(normalize_url(abs)?);
        } else {
            unreachable!()
        }
    }

    Ok(())
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
