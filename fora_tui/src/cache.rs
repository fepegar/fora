use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

/// A cached value with TTL-based expiry and forced refresh support.
#[derive(Debug)]
pub struct Cache<T> {
    inner: Arc<RwLock<CacheInner<T>>>,
    ttl: Duration,
}

#[derive(Debug)]
struct CacheInner<T> {
    data: Option<T>,
    last_fetched: Option<Instant>,
    loading: bool,
}

impl<T: Clone + Send + Sync + 'static> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                data: None,
                last_fetched: None,
                loading: false,
            })),
            ttl,
        }
    }

    /// Returns cached data if available.
    pub async fn get(&self) -> Option<T> {
        self.inner.read().await.data.clone()
    }

    /// Returns true if the cache is stale or empty.
    pub async fn is_stale(&self) -> bool {
        let inner = self.inner.read().await;
        match inner.last_fetched {
            None => true,
            Some(t) => t.elapsed() > self.ttl,
        }
    }

    /// Returns true if a fetch is currently in progress.
    pub async fn is_loading(&self) -> bool {
        self.inner.read().await.loading
    }

    /// Marks the cache as loading. Returns false if already loading.
    pub async fn start_loading(&self) -> bool {
        let mut inner = self.inner.write().await;
        if inner.loading {
            return false;
        }
        inner.loading = true;
        true
    }

    /// Updates the cached data and clears the loading flag.
    pub async fn set(&self, data: T) {
        let mut inner = self.inner.write().await;
        inner.data = Some(data);
        inner.last_fetched = Some(Instant::now());
        inner.loading = false;
    }

    /// Clears the cache, forcing the next check to be stale.
    pub async fn invalidate(&self) {
        let mut inner = self.inner.write().await;
        inner.last_fetched = None;
        inner.loading = false;
    }

    /// Returns a clone of the Arc for sharing across tasks.
    pub fn handle(&self) -> Cache<T> {
        Cache {
            inner: Arc::clone(&self.inner),
            ttl: self.ttl,
        }
    }
}

impl<T> Clone for Cache<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            ttl: self.ttl,
        }
    }
}
