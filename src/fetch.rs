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

    // Use simple sleep for rate-limiting for MVP
    tokio::time::sleep(tokio::time::Duration::from_millis(rand::random_range(
        200..500,
    )))
    .await;

    Ok(text)
}

pub(crate) async fn extract_links<R: UrlRepo>(
    document: &Html,
    repo: &mut R,
    base: &Url,
    anchor_selector: &Selector,
) -> Res<usize> {
    let mut res = 0usize;

    for url in document.select(anchor_selector) {
        if let Some(href) = url.attr("href") {
            let abs = base
                .join(href.trim_end_matches('/'))
                .with_context(|| format!("Failed to resolve relative URL: {}", href))?;

            repo.add(normalize_url(abs)?).await?;

            res += 1;
        } else {
            unreachable!()
        }
    }

    Ok(res)
}

pub(crate) fn extract_text(document: &Html, body_selector: &Selector) -> String {
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

pub(crate) fn extract_title(document: &Html, title_selector: &Selector) -> String {
    if let Some(title) = document.select(title_selector).next() {
        title.text().collect::<String>().trim().to_string()
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
    use scraper::{Html, Selector};
    use url::Url;

    use crate::{
        InMemoryRepo, UrlRepo,
        error::{Res, ResExt},
        fetch::{extract_links, extract_text, extract_title, normalize_url},
    };

    #[test]
    fn test_normalize_url() -> Res<()> {
        let url = url::Url::parse("http://ExAmPlE.com/index.html#section3")
            .context("Failed to parse URL for testing")?;

        let normalized = normalize_url(url).context("Failed to normalize URL")?;

        assert_eq!(normalized, "https://example.com/index.html");

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

        let mut repo = InMemoryRepo::new();

        let base = Url::parse("https://example.com/category/index.html")
            .context("Failed to parse base URL for testing resolving relative paths")?;

        let links = extract_links(&document, &mut repo, &base, &anchor_selector)
            .await
            .context("Failed to extract links")?;

        assert_eq!(links, 2);
        assert_eq!(
            repo.pop().await?.unwrap(),
            "https://example.com/category/path/to/page/index.html"
        );
        assert_eq!(
            repo.pop().await?.unwrap(),
            "https://example.com/path/to/another/page/index.html"
        );

        Ok(())
    }
}
