use std::hash::Hash;

use moka::future::{Cache, CacheBuilder};
use moka::notification::RemovalCause;

use super::MemoryCache;
use crate::metrics::record_cache_eviction;
use crate::settings::enabled_state::EnabledState;

pub struct MemoryCacheBuilder<K, V> {
    name: &'static str,
    metrics: EnabledState,
    inner: CacheBuilder<K, V, Cache<K, V>>,
}

impl<K, V> MemoryCacheBuilder<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub(super) fn new(name: &'static str) -> Self {
        Self {
            name,
            metrics: EnabledState::Enabled,
            inner: Cache::builder(),
        }
    }

    pub fn metrics(mut self, metrics: EnabledState) -> Self {
        self.metrics = metrics;

        self
    }

    pub fn configure(
        mut self,
        configure: impl FnOnce(CacheBuilder<K, V, Cache<K, V>>) -> CacheBuilder<K, V, Cache<K, V>>,
    ) -> Self {
        self.inner = configure(self.inner);

        self
    }

    pub fn build(self) -> MemoryCache<K, V> {
        let name = self.name;

        let inner = if self.metrics.is_enabled() {
            self.inner
                .eviction_listener(move |_, _, cause| {
                    record_cache_eviction(name, removal_cause(cause));
                })
                .build()
        } else {
            self.inner.build()
        };

        MemoryCache {
            name,
            metrics: self.metrics,
            inner,
        }
    }
}

fn removal_cause(cause: RemovalCause) -> &'static str {
    match cause {
        RemovalCause::Expired => "expired",
        RemovalCause::Explicit => "explicit",
        RemovalCause::Replaced => "replaced",
        RemovalCause::Size => "size",
    }
}
