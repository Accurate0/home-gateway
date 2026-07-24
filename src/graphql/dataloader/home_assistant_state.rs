use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct HomeAssistantStateDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct HomeAssistantStateModel {
    pub entity_id: String,
    pub state: String,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for HomeAssistantStateDataLoader {
    type Value = HomeAssistantStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            HomeAssistantStateModel,
            r#"
            SELECT entity_id, state, updated_at
            FROM latest_home_assistant_state
            WHERE entity_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-home-assistant-state"))
        .await?;

        Ok(rows.into_iter().map(|r| (r.entity_id.clone(), r)).collect())
    }
}
