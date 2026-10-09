use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::cache::MemoryCache;
use crate::device_registry::DeviceRegistry;
use crate::repo::DeviceRepo;
use crate::settings::LastSeenSettings;

#[derive(Clone)]
pub struct LastSeen {
    devices: DeviceRegistry,
    repo: DeviceRepo,
    seen: MemoryCache<String, DateTime<Utc>>,
    written: MemoryCache<String, ()>,
}

impl LastSeen {
    pub fn new(devices: DeviceRegistry, repo: DeviceRepo, settings: &LastSeenSettings) -> Self {
        let seen = MemoryCache::builder("device_last_seen")
            .configure(|cache| cache.max_capacity(settings.capacity))
            .build();

        let written = MemoryCache::builder("device_last_seen_written")
            .configure(|cache| {
                cache
                    .max_capacity(settings.capacity)
                    .time_to_live(settings.write_interval())
            })
            .build();

        Self {
            devices,
            repo,
            seen,
            written,
        }
    }

    #[tracing::instrument(
        name = "device.last_seen.lookup",
        skip_all,
        fields(keys = addresses.len(), cached = tracing::field::Empty),
        err
    )]
    pub async fn lookup(
        &self,
        addresses: &[String],
    ) -> Result<HashMap<String, DateTime<Utc>>, sqlx::Error> {
        let mut by_key: HashMap<String, Vec<String>> = HashMap::new();

        for address in addresses {
            let Some(device_key) = self.devices.watchdog_key(address) else {
                continue;
            };

            by_key
                .entry(device_key.to_owned())
                .or_default()
                .push(address.clone());
        }

        let mut seen: HashMap<String, DateTime<Utc>> = HashMap::new();
        let mut missing: Vec<String> = Vec::new();

        for device_key in by_key.keys() {
            match self.seen.get(device_key).await {
                Some(last_seen) => {
                    seen.insert(device_key.clone(), last_seen);
                }
                None => missing.push(device_key.clone()),
            }
        }

        tracing::Span::current().record("cached", seen.len());

        if !missing.is_empty() {
            for row in self.repo.last_seen_many(&missing).await? {
                let entry = self
                    .seen
                    .or_insert(row.device_key.clone(), row.last_seen)
                    .await;

                seen.insert(row.device_key, entry.into_value());
            }
        }

        Ok(seen
            .into_iter()
            .filter_map(|(device_key, last_seen)| Some((by_key.get(&device_key)?, last_seen)))
            .flat_map(|(addresses, last_seen)| {
                addresses
                    .iter()
                    .map(move |address| (address.clone(), last_seen))
            })
            .collect())
    }

    #[tracing::instrument(
        name = "device.last_seen.record",
        skip_all,
        fields(throttled = tracing::field::Empty)
    )]
    pub async fn record(&self, address: &str) {
        let Some(device_key) = self.devices.watchdog_key(address) else {
            tracing::warn!("no registered device for {address}, not recording last seen");
            return;
        };

        self.seen.insert(device_key.to_owned(), Utc::now()).await;

        let throttled = self.written.get(device_key).await.is_some();

        tracing::Span::current().record("throttled", throttled);

        if throttled {
            return;
        }

        match self.repo.touch_last_seen(device_key).await {
            Ok(()) => self.written.insert(device_key.to_owned(), ()).await,
            Err(e) => tracing::error!("failed to record last seen for {device_key}: {e}"),
        }
    }
}
