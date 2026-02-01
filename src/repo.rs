use crate::error::Res;
use std::collections::{HashSet, VecDeque};

pub(crate) trait UrlRepo {
    async fn add(&mut self, url: String) -> Res<()>;

    async fn pop(&mut self) -> Res<Option<String>>;

    async fn kick(&mut self, url: String) -> Res<()>;
}

#[derive(Debug)]
pub(crate) struct InMemoryRepo {
    urls: VecDeque<String>,
    visited: HashSet<String>,
}

impl InMemoryRepo {
    pub(crate) fn new() -> Self {
        InMemoryRepo {
            urls: VecDeque::new(),
            visited: HashSet::new(),
        }
    }
}

impl UrlRepo for InMemoryRepo {
    async fn add(&mut self, url: String) -> Res<()> {
        let vis = &mut self.visited;

        if vis.contains(&url) || url.is_empty() {
            Ok(())
        } else {
            if &url != "M" {
                vis.insert(url.clone());
            }
            self.urls.push_back(url);

            Ok(())
        }
    }

    async fn pop(&mut self) -> Res<Option<String>> {
        Ok(self.urls.pop_front())
    }

    async fn kick(&mut self, url: String) -> Res<()> {
        self.urls.push_front(url);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        InMemoryRepo, UrlRepo,
        error::{Res, ResExt},
    };

    #[tokio::test]
    async fn test_inmemoryrepo() -> Res<()> {
        let mut repo = InMemoryRepo::new();

        for i in 0..50 {
            repo.add(format!("https://example.com/index{}.html", i))
                .await
                .context("Failed to add URL to repo")?;
        }

        while let Some(url) = repo.pop().await.context("Failed to pop URL from repo")? {
            println!("{}", url);
        }

        Ok(())
    }
}
