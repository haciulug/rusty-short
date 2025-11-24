use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use crate::domain::Link;

#[derive(Clone)]
pub struct LinkCache {
    cache: Arc<Cache<String, Link>>,
}

impl LinkCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Link> {
        self.cache.get(key).await
    }

    pub async fn set(&self, key: String, link: Link) {
        self.cache.insert(key, link).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    pub fn size(&self) -> u64 {
        self.cache.entry_count()
    }
}

