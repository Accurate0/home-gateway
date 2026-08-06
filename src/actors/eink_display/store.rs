use super::{EInkDisplayActor, render::content_hash};
use crate::settings::EinkDisplaySettings;

impl EInkDisplayActor {
    pub(super) async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image_key: &str,
        image_content_hash: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, image_key, image_content_hash) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, image_content_hash = EXCLUDED.image_content_hash",
            device_id,
            name,
            image_key,
            image_content_hash,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        tracing::info!("eink display image updated for {device_id} -> {image_key}");

        Ok(())
    }

    pub(super) async fn store_next_wake(
        &self,
        device_id: &str,
        name: &str,
        next_wake_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, next_wake_at) VALUES ($1, $2, $3) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, next_wake_at = EXCLUDED.next_wake_at",
            device_id,
            name,
            next_wake_at,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    pub(super) async fn get_stored_next_wake(
        &self,
        device_id: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ractor::ActorProcessingErr> {
        let row = sqlx::query!(
            "SELECT next_wake_at FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(row.and_then(|row| row.next_wake_at))
    }

    pub(super) async fn stored_image_key(
        &self,
        device_id: &str,
    ) -> Result<Option<String>, ractor::ActorProcessingErr> {
        let row = sqlx::query!(
            "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(row.and_then(|row| row.image_key))
    }

    pub(super) async fn adopt_last_dashboard(
        &self,
        device_id: &str,
        display: &EinkDisplaySettings,
        image_key: &str,
    ) -> Result<bool, ractor::ActorProcessingErr> {
        if self.stored_image_key(device_id).await?.as_deref() == Some(image_key) {
            return Ok(true);
        }

        let image = match self.shared_actor_state.s3.get_object(image_key).await {
            Ok(image) => image,
            Err(e) => {
                tracing::warn!(
                    "no previous dashboard render at {image_key} for {device_id} ({e}), \
                     serving the current frame until the screenshot lands"
                );
                return Ok(false);
            }
        };

        self.store_render(device_id, &display.name, image_key, &content_hash(&image))
            .await?;

        Ok(true)
    }
}
