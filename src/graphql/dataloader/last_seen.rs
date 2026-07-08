use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct LastSeenDataLoader {
    pub database: Pool<Postgres>,
}

struct LastSeenRow {
    address: String,
    last_seen: DateTime<Utc>,
}

impl Loader<String> for LastSeenDataLoader {
    type Value = DateTime<Utc>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
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
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-last-seen"))
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.address, r.last_seen))
            .collect())
    }
}
