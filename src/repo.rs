use crate::error::Res;
use std::collections::{HashSet, VecDeque};

pub(crate) trait UrlRepo {
    fn add(&mut self, url: String) -> Res<()>;

    fn pop(&mut self) -> Res<Option<String>>;

    fn len(&self) -> Res<usize>;

    fn crawled_len(&self) -> Res<usize>;
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
    fn add(&mut self, url: String) -> Res<()> {
        if self.visited.contains(&url) || url.is_empty() {
            Ok(())
        } else {
            self.visited.insert(url.clone());

            self.urls.push_back(url);

            Ok(())
        }
    }

    fn pop(&mut self) -> Res<Option<String>> {
        Ok(self.urls.pop_front())
    }

    fn len(&self) -> Res<usize> {
        Ok(self.urls.len())
    }

    fn crawled_len(&self) -> Res<usize> {
        Ok(self.visited.len())
    }
}
