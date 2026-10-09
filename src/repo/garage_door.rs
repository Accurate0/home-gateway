use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    async_graphql::Enum,
    Eq,
    Copy,
    schemars::JsonSchema,
    strum::Display,
)]
#[sqlx(type_name = "garage_door_state", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum GarageDoorState {
    Open,
    Opening,
    Closed,
    Closing,
}

#[derive(Clone)]
pub struct GarageDoorRepo {
    db: Pool<Postgres>,
}

#[derive(Clone, Debug)]
pub struct GarageDoorStateRow {
    pub device_id: String,
    pub state: GarageDoorState,
    pub contact: Option<bool>,
    pub changed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GarageDoorRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.garage_door.record", err)]
    pub async fn record(
        &self,
        event_id: Uuid,
        device_id: &str,
        state: GarageDoorState,
        contact: Option<bool>,
    ) -> Result<Option<GarageDoorState>, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let previous = sqlx::query_scalar!(
            r#"SELECT state AS "state: GarageDoorState" FROM latest_garage_door_state WHERE device_id = $1 FOR UPDATE"#,
            device_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO garage_door_events (event_id, device_id, state, contact) VALUES ($1, $2, $3, $4)",
            event_id,
            device_id,
            state as GarageDoorState,
            contact,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_garage_door_state (device_id, state, contact) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (device_id) DO UPDATE SET \
               changed_at = CASE WHEN latest_garage_door_state.state = EXCLUDED.state \
                 THEN latest_garage_door_state.changed_at ELSE now() END, \
               state = EXCLUDED.state, \
               contact = COALESCE(EXCLUDED.contact, latest_garage_door_state.contact), \
               updated_at = now()",
            device_id,
            state as GarageDoorState,
            contact,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(previous)
    }

    #[tracing::instrument(skip_all, name = "db.garage_door.closed_since", err)]
    pub async fn closed_since(
        &self,
        device_id: &str,
        since: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let closed = sqlx::query_scalar!(
            r#"SELECT EXISTS (
                 SELECT 1 FROM garage_door_events
                 WHERE device_id = $1 AND state = 'closed' AND "time" > $2
               ) AS "closed!""#,
            device_id,
            since,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(closed)
    }

    #[tracing::instrument(skip_all, name = "db.garage_door.latest", err)]
    pub async fn latest(&self, device_id: &str) -> Result<Option<GarageDoorStateRow>, sqlx::Error> {
        sqlx::query_as!(
            GarageDoorStateRow,
            r#"
            SELECT device_id, state AS "state: GarageDoorState", contact, changed_at, updated_at
            FROM latest_garage_door_state
            WHERE device_id = $1
            "#,
            device_id
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.garage_door.latest_many", fields(keys = keys.len()), err)]
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<GarageDoorStateRow>, sqlx::Error> {
        sqlx::query_as!(
            GarageDoorStateRow,
            r#"
            SELECT device_id, state AS "state: GarageDoorState", contact, changed_at, updated_at
            FROM latest_garage_door_state
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }
}
