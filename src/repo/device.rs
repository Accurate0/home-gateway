use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct DeviceRepo {
    db: Pool<Postgres>,
}

pub struct LastSeenRow {
    pub device_key: String,
    pub last_seen: DateTime<Utc>,
}

pub struct DeviceConnectionRow {
    pub device_id: String,
    pub connected: bool,
    pub changed_at: DateTime<Utc>,
}

impl DeviceRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.device.touch_last_seen", err)]
    pub async fn touch_last_seen(&self, device_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO device_last_seen (device_key, last_seen) VALUES ($1, now()) \
             ON CONFLICT (device_key) DO UPDATE SET last_seen = now()",
            device_key
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.device.upsert_known", err)]
    pub async fn upsert_known(&self, ieee_addr: &str, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO known_devices (ieee_addr, name) VALUES ($1, $2) ON CONFLICT (ieee_addr) DO UPDATE SET name = $2",
            ieee_addr,
            name
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    #[tracing::instrument(skip_all, name = "db.device.last_seen_many", fields(keys = keys.len()), err)]
    pub async fn last_seen_many(&self, keys: &[String]) -> Result<Vec<LastSeenRow>, sqlx::Error> {
        sqlx::query_as!(
            LastSeenRow,
            r#"
            SELECT device_key AS "device_key!", last_seen AS "last_seen!"
            FROM device_last_seen
            WHERE device_key = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.device.record_connection", err)]
    pub async fn record_connection(
        &self,
        device_id: &str,
        connected: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO device_connection (device_id, connected, changed_at) VALUES ($1, $2, now()) \
             ON CONFLICT (device_id) DO UPDATE SET connected = $2, changed_at = now()",
            device_id,
            connected
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.device.connections", err)]
    pub async fn connections(&self) -> Result<Vec<DeviceConnectionRow>, sqlx::Error> {
        sqlx::query_as!(
            DeviceConnectionRow,
            "SELECT device_id, connected, changed_at FROM device_connection"
        )
        .fetch_all(&self.db)
        .await
    }
}
