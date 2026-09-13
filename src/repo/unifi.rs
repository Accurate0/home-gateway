use crate::db::UnifiState;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub struct UnifiClientStateRow {
    pub name: String,
    pub state: UnifiState,
    pub time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct UnifiRepo {
    db: Pool<Postgres>,
}

impl UnifiRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.unifi.upsert_state", err)]
    pub async fn upsert_state(
        &self,
        mac_address: &str,
        name: Option<&str>,
        hostname: Option<&str>,
        state: UnifiState,
    ) -> Result<Option<UnifiState>, sqlx::Error> {
        let row = sqlx::query!(
            r#"WITH previous AS (
                 SELECT state FROM unifi_client_state WHERE mac_address = $1
               )
               INSERT INTO unifi_client_state (mac_address, name, hostname, state)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (mac_address) DO UPDATE SET
                 name = COALESCE(EXCLUDED.name, unifi_client_state.name),
                 hostname = COALESCE(EXCLUDED.hostname, unifi_client_state.hostname),
                 state = EXCLUDED.state,
                 updated_at = CASE
                   WHEN unifi_client_state.state = EXCLUDED.state THEN unifi_client_state.updated_at
                   ELSE now()
                 END
               RETURNING (SELECT state FROM previous) AS "previous: UnifiState""#,
            mac_address,
            name,
            hostname,
            state as UnifiState
        )
        .fetch_one(&self.db)
        .await?;

        Ok(row.previous)
    }

    #[tracing::instrument(skip_all, name = "db.unifi.latest_states", err)]
    pub async fn latest_states(&self) -> Result<Vec<UnifiClientStateRow>, sqlx::Error> {
        sqlx::query_as!(
            UnifiClientStateRow,
            r#"SELECT COALESCE(name, hostname, mac_address) AS "name!",
                      state AS "state: UnifiState",
                      updated_at AS "time"
               FROM unifi_client_state
               ORDER BY 1"#
        )
        .fetch_all(&self.db)
        .await
    }
}
