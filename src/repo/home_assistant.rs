use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct HomeAssistantRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct HomeAssistantStateRow {
    pub entity_id: String,
    pub state: String,
    pub updated_at: DateTime<Utc>,
}

pub struct HomeAssistantEventRow {
    pub event_id: Uuid,
    pub entity_id: String,
    pub state: String,
    pub updated_at: DateTime<Utc>,
}

impl HomeAssistantRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn append_event(
        &self,
        event_id: Uuid,
        entity_id: &str,
        state: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO home_assistant_events (event_id, entity_id, state) VALUES ($1, $2, $3)",
            event_id,
            entity_id,
            state,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn upsert_latest(
        &self,
        event_id: Uuid,
        entity_id: &str,
        state: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO latest_home_assistant_state (entity_id, state, event_id, updated_at) VALUES ($1, $2, $3, now())
            ON CONFLICT (entity_id)
            DO UPDATE SET
                state = EXCLUDED.state,
                event_id = EXCLUDED.event_id,
                updated_at = EXCLUDED.updated_at
            "#,
            entity_id,
            state,
            event_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<HomeAssistantStateRow>, sqlx::Error> {
        sqlx::query_as!(
            HomeAssistantStateRow,
            r#"
            SELECT entity_id, state, updated_at
            FROM latest_home_assistant_state
            WHERE entity_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-home-assistant-state"))
        .await
    }
    pub async fn latest_all(&self) -> Result<Vec<HomeAssistantEventRow>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"SELECT event_id, entity_id, state, updated_at FROM latest_home_assistant_state ORDER BY updated_at DESC"#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| HomeAssistantEventRow {
                event_id: row.event_id,
                entity_id: row.entity_id,
                state: row.state,
                updated_at: row.updated_at,
            })
            .collect())
    }
}
