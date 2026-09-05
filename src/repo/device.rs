use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;

#[derive(Clone)]
pub struct DeviceRepo {
    db: Pool<Postgres>,
}

pub struct LastSeenRow {
    pub address: String,
    pub last_seen: DateTime<Utc>,
}

impl DeviceRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

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
    pub async fn last_seen_many(&self, keys: &[String]) -> Result<Vec<LastSeenRow>, sqlx::Error> {
        sqlx::query_as!(
            LastSeenRow,
            r#"
            SELECT kd.ieee_addr AS "address!", dls.last_seen AS "last_seen!"
            FROM device_last_seen dls
            JOIN known_devices kd ON kd.name = dls.device_key
            WHERE kd.ieee_addr = ANY($1)
            UNION
            SELECT dls.device_key AS "address!", dls.last_seen AS "last_seen!"
            FROM device_last_seen dls
            WHERE dls.device_key = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-last-seen"))
        .await
    }
}
