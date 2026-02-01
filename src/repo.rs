use tokio::sync::Mutex;

use crate::error::Res;
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

pub(crate) trait UrlRepo {
    async fn add(&mut self, url: String) -> Res<()>;

    async fn pop(&mut self) -> Res<Option<String>>;
}

#[derive(Debug)]
pub(crate) struct InMemoryRepo {
    urls: Arc<Mutex<VecDeque<String>>>,
    visited: Arc<Mutex<HashSet<String>>>,
}

impl InMemoryRepo {
    pub(crate) fn new() -> Self {
        InMemoryRepo {
            urls: Arc::new(Mutex::new(VecDeque::new())),
            visited: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl UrlRepo for InMemoryRepo {
    async fn add(&mut self, url: String) -> Res<()> {
        let temp_vis = Arc::clone(&self.visited);
        let mut vis = temp_vis.lock().await;

        if vis.contains(&url) || url.is_empty() {
            Ok(())
        } else {
            vis.insert(url.clone());
            drop(vis);
            drop(temp_vis);

            {
                let temp = Arc::clone(&self.urls);
                let mut queue = temp.lock().await;
                queue.push_back(url);
            }

            Ok(())
        }
    }

    async fn pop(&mut self) -> Res<Option<String>> {
        Ok(Arc::clone(&self.urls).lock().await.pop_front())
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
