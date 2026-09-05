use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;

pub struct StoredRender {
    pub image_key: Option<String>,
    pub image_content_hash: Option<String>,
}

#[derive(Clone)]
pub struct EinkRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct EinkDisplayRow {
    pub device_id: String,
    pub battery_voltage: Option<f64>,
    pub is_charging: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

impl EinkRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image_key: &str,
        content_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, image_key, image_content_hash) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, image_content_hash = EXCLUDED.image_content_hash",
            device_id,
            name,
            image_key,
            content_hash,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn store_seen(&self, device_id: &str, name: &str) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
        next_wake_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT next_wake_at FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.and_then(|row| row.next_wake_at))
    }

    pub async fn stored_render(
        &self,
        device_id: &str,
    ) -> Result<Option<StoredRender>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| StoredRender {
            image_key: row.image_key,
            image_content_hash: row.image_content_hash,
        }))
    }

    pub async fn partial_refresh_count(&self, device_id: &str) -> Result<i32, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT partial_refresh_count FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(count.unwrap_or(0))
    }

    pub async fn increment_partial_refresh_count(
        &self,
        device_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE eink_display SET partial_refresh_count = partial_refresh_count + 1 WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn reset_partial_refresh_count(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE eink_display SET partial_refresh_count = 0 WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn battery_many(&self, keys: &[String]) -> Result<Vec<EinkDisplayRow>, sqlx::Error> {
        sqlx::query_as!(
            EinkDisplayRow,
            r#"
            SELECT device_id, battery_voltage, is_charging, updated_at
            FROM eink_display
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-eink-display"))
        .await
    }
}
