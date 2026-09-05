use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

pub struct LastSeenRow {
    pub last_seen: DateTime<Utc>,
    pub alerted_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct WatchdogRepo {
    db: Pool<Postgres>,
}

impl WatchdogRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn last_seen(&self) -> Result<HashMap<String, LastSeenRow>, sqlx::Error> {
        let rows = sqlx::query!("SELECT device_key, last_seen, alerted_at FROM device_last_seen")
            .fetch_all(&self.db)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.device_key,
                    LastSeenRow {
                        last_seen: row.last_seen,
                        alerted_at: row.alerted_at,
                    },
                )
            })
            .collect())
    }

    pub async fn mark_alerted(
        &self,
        device_key: &str,
        at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE device_last_seen SET alerted_at = $1 WHERE device_key = $2",
            at,
            device_key
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn clear_alerted(&self, device_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE device_last_seen SET alerted_at = NULL WHERE device_key = $1",
            device_key
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
