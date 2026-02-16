use std::{sync::Arc, time::Duration};

use owo_colors::OwoColorize;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    crawler::CrawnClient,
    error::{Res, ResErr, ResExt},
};

pub async fn fetch_url(url: &String, client: Arc<CrawnClient>) -> Res<String> {
    let res = client.get(url).await?;
    let stat = res.status();

    if !stat.is_success() {
        if let StatusCode::TOO_MANY_REQUESTS = stat {
            client.timeout(Duration::from_millis(2500)).await;
            res.error_for_status_ref()
                .with_context(format_args!(
                    "Failed to fetch URL: {}",
                    url.bright_blue().italic()
                ))
                .with_context(format_args!(
                    "Server returned {} response, status code: {}",
                    "`TOO_MANY_REQUESTS`".yellow(),
                    "429".red().bold()
                ))
                .context(
                    "Will wait for 2.5 second timeout to avoid more bad responses and IP bans",
                )?;
        } else {
            res.error_for_status_ref()
                .with_context(format_args!(
                    "Failed to fetch URL: {}",
                    url.bright_blue().italic()
                ))
                .with_context(format_args!(
                    "Server returned status code: {}",
                    stat.as_str().red().bold()
                ))?;
        }
    }
    let text = res.text().await.with_context(format_args!(
        "Failed to fetch HTML (content) from URL: {}",
        url.bright_blue().italic()
    ))?;

    Ok(text)
}

pub fn extract_links(document: &Html, base: Arc<Url>, anchor_selector: &Selector) -> Vec<Res<Url>> {
    document
        .select(anchor_selector)
        .map(|anchor| {
            let href = anchor.attr("href").ok_or_else(|| {
                ResErr::new(
                    "Failed to extract URL from HTML anchor tag (link)",
                    String::from("Failed to select 'href' from anchor tag"),
                )
            })?;

            base.join(href).with_context(format_args!(
                "Failed to resolve relative URL: {}",
                href.bright_blue().italic()
            ))
        })
        .collect()
}
pub fn extract_text(document: &Html, body_selector: &Selector) -> String {
    if let Some(body) = document.select(body_selector).next() {
        body.text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

pub fn extract_title(document: &Html, title_selector: &Selector) -> String {
    if let Some(title) = document.select(title_selector).next() {
        title.text().collect::<String>().trim().to_string()
    } else {
        String::new()
    }
}

pub fn normalize_url(mut url: Url) -> Res<String> {
    if let Some(domain) = url.domain() {
        let res = url.set_host(Some(&domain.to_lowercase()));
        res.context("Failed to set host domain for URL")?;
    } else {
        return Err(url::ParseError::EmptyHost).context(
            "Failed to normalize host domain for URL as it does not contain a valid host domain",
        );
    }

    url.set_fragment(None);

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use scraper::{Html, Selector};
    use url::Url;

    use crate::{
        error::{Res, ResExt},
        fetch::{extract_links, extract_text, extract_title, normalize_url},
    };

    #[test]
    fn test_normalize_url() -> Res<()> {
        let url = url::Url::parse("http://ExAmPlE.com/index.html#section3")
            .context("Failed to parse URL for testing")?;

        let normalized = normalize_url(url).context("Failed to normalize URL")?;

        assert_eq!(normalized, "http://example.com/index.html");

        Ok(())
    }

    #[test]
    fn test_extract_title() -> Res<()> {
        let html = Html::parse_document(
            r#"
<html>
  <head>
    <title>     Example title for test       </title>
  </head>
</html>
            "#,
        );

        let title_selector =
            Selector::parse("title").context("Failed to parse selector for HTML title tag")?;

        let title = extract_title(&html, &title_selector);

        assert_eq!(title, "Example title for test");

        Ok(())
    }

    #[test]
    fn test_extract_text() -> Res<()> {
        let document = Html::parse_document(
            r#"
<html>
  <body>
            Example body  text for     test   
  </body>
</html>
            "#,
        );

        let body_selector =
            Selector::parse("body").context("Failed to parse selector for HTML body tag")?;

        let text = extract_text(&document, &body_selector);

        assert_eq!(text, "Example body text for test");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_links() -> Res<()> {
        let document = Html::parse_document(
            r#"
<html>
  <body>
    <a href="path/to/page/index.html">link</a>
    <a href="/path/to/another/page/index.html">link</a>
  </body>
</html>
            "#,
        );

        let anchor_selector =
            Selector::parse("a[href]").context("Failed to parse selector for HTML anchor tag")?;

        let base = Url::parse("https://example.com/category/index.html")
            .context("Failed to parse base URL for testing resolving relative paths")?;

        let links = extract_links(&document, Arc::new(base), &anchor_selector);

        assert_eq!(
            links
                .iter()
                .map(move |link| link.as_ref().unwrap().clone())
                .collect::<Vec<Url>>(),
            vec![
                Url::parse("https://example.com/category/path/to/page/index.html").unwrap(),
                Url::parse("https://example.com/path/to/another/page/index.html").unwrap()
            ]
        );

        Ok(())
    }
}
