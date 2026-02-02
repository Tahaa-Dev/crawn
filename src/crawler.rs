use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use owo_colors::OwoColorize;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use tokio::{sync::Mutex, time::sleep};
use url::Url;

use crate::{
    UrlRepo,
    error::{Log, Res, ResExt},
    fetch::*,
    match_option,
    output::write_output,
};

pub(crate) struct CrawnClient {
    client: Client,
    next_req: Mutex<Instant>,
}

impl CrawnClient {
    pub(crate) fn new() -> Res<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .context("Failed to build client")?,

            next_req: Mutex::new(Instant::now()),
        })
    }

    pub(crate) async fn get(&self, url: &str) -> Res<Response> {
        let mut next_req = self.next_req.lock().await;

        let now = Instant::now();
        if now < *next_req {
            sleep(*next_req - now).await;
        }

        let res = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch URL: {}", url.bright_blue().italic()));

        *next_req = Instant::now() + Duration::from_millis(rand::random_range(300..=600));

        res
    }

    pub(crate) async fn timeout(&self, time: Duration) {
        *self.next_req.lock().await = Instant::now() + time;
    }
}

pub(crate) struct Selectors {
    pub(crate) anchor: Selector,
    pub(crate) title: Selector,
    pub(crate) body: Option<Selector>,
}

pub(crate) async fn worker<R: UrlRepo>(
    repo: Arc<Mutex<R>>,
    selectors: Arc<Selectors>,
    client: Arc<CrawnClient>,
    url: String,
    can_extract: bool,
) -> Res<()> {
    let args = &*crate::ARGS;
    let client = Arc::clone(&client);

    let base = Url::parse(&url)
        .with_context(|| format!("Failed to parse URL: {}", &url.italic().bright_blue()))?;

    let content = fetch_url(&url, client).await?;

    if args.verbose {
        format!("Fetched content from URL: {}", &url.bright_blue().italic())
            .log("[INFO]")
            .await?;
    }

    let (links, title, text, content) = {
        let selectors = Arc::clone(&selectors);
        let mut link_count = 0usize;

        let task = tokio::task::spawn_blocking(move || {
            let doc = Html::parse_document(&content);
            let links = if can_extract {
                extract_links(&doc, Arc::new(base), &selectors.anchor)
            } else {
                Vec::new()
            };

            let text = selectors
                .body
                .as_ref()
                .map(|body_selector| extract_text(&doc, body_selector));
            let title = extract_title(&doc, &selectors.title);

            (
                text,
                title,
                links,
                if args.include_content {
                    Some(content)
                } else {
                    None
                },
            )
        });

        let (text, title, links, content) = task
            .await
            .context("Failed to extract links and text from HTML body concurrently")?;

        {
            let temp = Arc::clone(&repo);
            let mut rp = temp.lock().await;

            for link in links {
                let link = match_option!(link.log("[WARN]").await?);
                let link = match_option!(normalize_url(link).log("[WARN]").await?);

                match_option!(rp.add(link).await.log("[WARN]").await?);

                link_count += 1;
            }
        }

        (link_count, title, text, content)
    };

    write_output(url, title, links, text, content)
        .await
        .context("Failed to write output entry for URL")?;

    Ok(())
}

static GENERICS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["tutorial", "guide", "blog"]));

pub(crate) fn should_crawl(
    base_domain: Arc<String>,
    base_keywords: Arc<HashSet<String>>,
    other: &Url,
) -> bool {
    if let Some(other_domain) = other.domain() {
        if other_domain != base_domain.as_str() {
            return false;
        }
    } else {
        return false;
    }

    let other_keywords = get_keywords(other);

    let match_count = other_keywords
        .iter()
        .filter(|kw| base_keywords.contains(kw.as_str()) || GENERICS.contains(&kw.as_str()))
        .count();

    if match_count >= 2 {
        return true;
    }

    true
}

// common stop words
static STOP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "how",
        "to",
        "the",
        "and",
        "for",
        "with",
        "from",
        "about",
        "by",
        "category",
        "catalogue",
    ])
});

pub(crate) fn get_keywords(url: &Url) -> HashSet<String> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use url::Url;

    use crate::{
        crawler::get_keywords,
        error::{Res, ResExt},
    };

    #[test]
    fn test_keyword_extraction() -> Res<()> {
        let url = Url::parse(
            "https://example.com/rust-programming-language/category/async/tokio/beginner_tutorial",
        )
        .context("Failed to parse URL")?;

        let kws = get_keywords(&url);

        assert_eq!(
            kws,
            HashSet::from([
                "rust".to_string(),
                "programming".to_string(),
                "language".to_string(),
                "async".to_string(),
                "tokio".to_string(),
                "beginner".to_string(),
                "tutorial".to_string()
            ])
        );

        Ok(())
    }
}
