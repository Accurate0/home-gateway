use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct ValetudoStateDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct ValetudoStateModel {
    pub identifier: String,
    pub state: Option<String>,
    pub battery_level: Option<i32>,
    pub fan_speed: Option<String>,
    pub current_clean_area: Option<f64>,
    pub clean_count: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for ValetudoStateDataLoader {
    type Value = ValetudoStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            ValetudoStateModel,
            r#"
            SELECT identifier, state, battery_level, fan_speed, current_clean_area, clean_count, updated_at
            FROM latest_valetudo_state
            WHERE identifier = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-valetudo-state"))
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.identifier.clone(), r))
            .collect())
    }
}
