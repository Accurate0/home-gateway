use super::EinkDisplayManager;
use super::source::SourceImage;
use crate::error::AppError;

impl EinkDisplayManager {
    pub async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image: &SourceImage,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, image_key, image_content_hash) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, image_content_hash = EXCLUDED.image_content_hash",
            device_id,
            name,
            image.image_key,
            image.content_hash,
        )
        .execute(&self.db)
        .await?;

        tracing::info!(
            "eink display image updated for {device_id} -> {}",
            image.image_key
        );

        Ok(())
    }

    pub async fn store_seen(&self, device_id: &str, name: &str) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, updated_at) VALUES ($1, $2, now()) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at",
            device_id,
            name,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn store_battery(
        &self,
        device_id: &str,
        name: &str,
        battery_voltage: f64,
        is_charging: Option<bool>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, battery_voltage, is_charging, updated_at) VALUES ($1, $2, $3, $4, now()) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, is_charging = EXCLUDED.is_charging, updated_at = EXCLUDED.updated_at",
            device_id,
            name,
            battery_voltage,
            is_charging,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn store_next_wake(
        &self,
        device_id: &str,
        name: &str,
        next_wake_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, next_wake_at) VALUES ($1, $2, $3) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, next_wake_at = EXCLUDED.next_wake_at",
            device_id,
            name,
            next_wake_at,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn stored_next_wake(
        &self,
        device_id: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
        let row = sqlx::query!(
            "SELECT next_wake_at FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.and_then(|row| row.next_wake_at))
    }

    pub async fn stored_render(&self, device_id: &str) -> Option<SourceImage> {
        let row = sqlx::query!(
            "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
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
