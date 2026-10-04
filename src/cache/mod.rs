mod builder;

use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use moka::Entry;
use moka::future::Cache;

use crate::metrics::{record_cache_request, record_cache_size};
use crate::settings::enabled_state::EnabledState;

pub use builder::MemoryCacheBuilder;

pub struct MemoryCache<K, V> {
    name: &'static str,
    metrics: EnabledState,
    inner: Cache<K, V>,
}

impl<K, V> Clone for MemoryCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            metrics: self.metrics,
            inner: self.inner.clone(),
        }
    }
}

impl<K, V> MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn builder(name: &'static str) -> MemoryCacheBuilder<K, V> {
        MemoryCacheBuilder::new(name)
    }

    pub async fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let value = self.inner.get(key).await;

        self.record_request(value.is_some());

        value
    }

    pub async fn insert(&self, key: K, value: V) {
        self.inner.insert(key, value).await;

        self.record_size();
    }

    pub async fn invalidate<Q>(&self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.invalidate(key).await;

        self.record_size();
    }

    pub async fn or_insert(&self, key: K, value: V) -> Entry<K, V> {
        let entry = self.inner.entry(key).or_insert(value).await;

        self.record_request(!entry.is_fresh());

        entry
    }

    pub async fn or_insert_with(&self, key: K, init: impl Future<Output = V>) -> Entry<K, V> {
        let entry = self.inner.entry(key).or_insert_with(init).await;

        self.record_request(!entry.is_fresh());

        entry
    }

    pub async fn or_try_insert_with<E>(
        &self,
        key: K,
        init: impl Future<Output = Result<V, E>>,
    ) -> Result<Entry<K, V>, Arc<E>>
    where
        E: Send + Sync + 'static,
    {
        let entry = self.inner.entry(key).or_try_insert_with(init).await;

        self.record_request(entry.as_ref().is_ok_and(|entry| !entry.is_fresh()));

        entry
    }

    fn record_request(&self, hit: bool) {
        if !self.metrics.is_enabled() {
            return;
        }

        record_cache_request(self.name, hit);

        self.record_size();
    }

    fn record_size(&self) {
        if !self.metrics.is_enabled() {
            return;
        }

        record_cache_size(
            self.name,
            self.inner.entry_count(),
            self.inner.weighted_size(),
            self.inner.policy().max_capacity(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(metrics: EnabledState) -> MemoryCache<String, u32> {
        MemoryCache::builder("test")
            .metrics(metrics)
            .configure(|cache| cache.max_capacity(8))
            .build()
    }

    #[tokio::test]
    async fn a_value_is_returned_once_inserted() {
        let cache = cache(EnabledState::Enabled);

        assert_eq!(cache.get("a").await, None);

        cache.insert("a".to_owned(), 1).await;

        assert_eq!(cache.get("a").await, Some(1));
    }

    #[tokio::test]
    async fn an_invalidated_value_is_gone() {
        let cache = cache(EnabledState::Enabled);

        cache.insert("a".to_owned(), 1).await;
        cache.invalidate("a").await;

        assert_eq!(cache.get("a").await, None);
    }

    #[tokio::test]
    async fn or_try_insert_with_is_fresh_only_the_first_time() {
        let cache = cache(EnabledState::Enabled);

        let first = cache
            .or_try_insert_with("a".to_owned(), async { Ok::<_, std::io::Error>(1) })
            .await
            .unwrap();

        let second = cache
            .or_try_insert_with("a".to_owned(), async { Ok::<_, std::io::Error>(2) })
            .await
            .unwrap();

        assert!(first.is_fresh());
        assert!(!second.is_fresh());
        assert_eq!(second.into_value(), 1);
    }

    #[tokio::test]
    async fn a_failed_init_caches_nothing() {
        let cache = cache(EnabledState::Enabled);

        let failed = cache
            .or_try_insert_with("a".to_owned(), async {
                Err::<u32, _>(std::io::Error::other("boom"))
            })
            .await;

        assert!(failed.is_err());
        assert_eq!(cache.get("a").await, None);
    }

    #[tokio::test]
    async fn or_insert_keeps_the_existing_value() {
        let cache = cache(EnabledState::Enabled);

        assert!(cache.or_insert("a".to_owned(), 1).await.is_fresh());
        assert!(!cache.or_insert("a".to_owned(), 2).await.is_fresh());

        let existing = cache.or_insert_with("a".to_owned(), async { 3 }).await;

        assert_eq!(existing.into_value(), 1);
    }

    #[tokio::test]
    async fn the_configure_closure_reaches_the_inner_cache() {
        let cache = cache(EnabledState::Enabled);

        assert_eq!(cache.inner.policy().max_capacity(), Some(8));
    }

    #[tokio::test]
    async fn a_cache_without_metrics_still_caches() {
        let cache = cache(EnabledState::Disabled);

        cache.insert("a".to_owned(), 1).await;

        assert_eq!(cache.get("a").await, Some(1));
    }
}
