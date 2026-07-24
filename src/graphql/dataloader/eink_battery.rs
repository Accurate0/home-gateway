use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct EinkDisplayDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct EinkDisplayModel {
    pub device_id: String,
    pub battery_voltage: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for EinkDisplayDataLoader {
    type Value = EinkDisplayModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            EinkDisplayModel,
            r#"
            SELECT device_id, battery_voltage, updated_at
            FROM eink_display
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-eink-display"))
        .await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
