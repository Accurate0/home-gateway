use crate::device_registry::DeviceRegistry;
use crate::repo::DeviceRepo;

pub async fn record(devices: &DeviceRegistry, repo: &DeviceRepo, address: &str) {
    let Some(device_key) = devices.watchdog_key(address) else {
        tracing::warn!("no registered device for {address}, not recording last seen");
        return;
    };

    if let Err(e) = repo.touch_last_seen(device_key).await {
        tracing::error!("failed to record last seen for {device_key}: {e}");
    }
}
