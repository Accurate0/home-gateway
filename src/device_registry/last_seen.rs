use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::device_registry::DeviceRegistry;
use crate::repo::DeviceRepo;

pub async fn lookup(
    devices: &DeviceRegistry,
    repo: &DeviceRepo,
    addresses: &[String],
) -> Result<HashMap<String, DateTime<Utc>>, sqlx::Error> {
    let mut by_key: HashMap<String, Vec<String>> = HashMap::new();

    for address in addresses {
        let Some(device_key) = devices.watchdog_key(address) else {
            continue;
        };

        by_key
            .entry(device_key.to_owned())
            .or_default()
            .push(address.clone());
    }

    let keys: Vec<String> = by_key.keys().cloned().collect();
    let rows = repo.last_seen_many(&keys).await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| Some((by_key.get(&row.device_key)?, row.last_seen)))
        .flat_map(|(addresses, last_seen)| {
            addresses
                .iter()
                .map(move |address| (address.clone(), last_seen))
        })
        .collect())
}

pub async fn record(devices: &DeviceRegistry, repo: &DeviceRepo, address: &str) {
    let Some(device_key) = devices.watchdog_key(address) else {
        tracing::warn!("no registered device for {address}, not recording last seen");
        return;
    };

    if let Err(e) = repo.touch_last_seen(device_key).await {
        tracing::error!("failed to record last seen for {device_key}: {e}");
    }
}
