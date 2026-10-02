use super::EinkDisplayManager;
use super::source::SourceImage;
use crate::error::AppError;
use crate::repo::eink::WakeRecord;

impl EinkDisplayManager {
    pub async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image: &SourceImage,
    ) -> Result<(), AppError> {
        self.eink
            .store_render(device_id, name, &image.image_key, &image.content_hash)
            .await?;

        tracing::info!(
            "eink display image updated for {device_id} -> {}",
            image.image_key
        );

        Ok(())
    }

    pub async fn store_seen(
        &self,
        device_id: &str,
        name: &str,
        trace_id: Option<&str>,
    ) -> Result<(), AppError> {
        self.eink.store_seen(device_id, name, trace_id).await?;

        Ok(())
    }

    pub async fn store_battery(
        &self,
        device_id: &str,
        name: &str,
        battery_voltage: f64,
        is_charging: Option<bool>,
    ) -> Result<(), AppError> {
        self.eink
            .store_battery(device_id, name, battery_voltage, is_charging)
            .await?;

        Ok(())
    }

    pub async fn store_next_wake(
        &self,
        device_id: &str,
        name: &str,
        next_wake_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        self.eink
            .store_next_wake(device_id, name, next_wake_at)
            .await?;

        Ok(())
    }

    pub async fn stored_next_wake(
        &self,
        device_id: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
        let next_wake_at = self.eink.stored_next_wake(device_id).await?;

        Ok(next_wake_at)
    }

    pub async fn displayed_hash(&self, device_id: &str) -> Option<String> {
        self.eink
            .displayed_hash(device_id)
            .await
            .inspect_err(
                |e| tracing::warn!(device_id = %device_id, "failed to read the displayed hash: {e}"),
            )
            .ok()
            .flatten()
    }

    pub async fn store_wake(
        &self,
        device_id: &str,
        name: &str,
        wake: &WakeRecord<'_>,
    ) -> Result<(), AppError> {
        self.eink.store_wake(device_id, name, wake).await?;

        Ok(())
    }

    pub async fn rtc_synced_at(&self, device_id: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        self.eink
            .rtc_synced_at(device_id)
            .await
            .inspect_err(
                |e| tracing::warn!(device_id = %device_id, "failed to read the last rtc sync: {e}"),
            )
            .ok()
            .flatten()
    }

    pub async fn stored_render(&self, device_id: &str) -> Option<SourceImage> {
        let row = self
            .eink
            .stored_render(device_id)
            .await
            .inspect_err(
                |e| tracing::warn!(device_id = %device_id, "failed to read the stored render: {e}"),
            )
            .ok()
            .flatten()?;

        Some(SourceImage {
            image_key: row.image_key?,
            content_hash: row.image_content_hash.unwrap_or_default(),
        })
    }
}
