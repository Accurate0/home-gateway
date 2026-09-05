use crate::db::UnifiState;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct UnifiClientMapping {
    pub name: String,
    pub id: Uuid,
}

#[derive(Clone)]
pub struct UnifiRepo {
    db: Pool<Postgres>,
}

impl UnifiRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn mapping_for(
        &self,
        mac_address: &str,
    ) -> Result<Option<UnifiClientMapping>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT name, id FROM unifi_clients_mapping WHERE mac_address = $1",
            mac_address
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| UnifiClientMapping {
            name: row.name,
            id: row.id,
        }))
    }

    pub async fn append_event(
        &self,
        event_id: Uuid,
        name: &str,
        id: &str,
        state: UnifiState,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO unifi_clients (event_id, name, id, state) VALUES ($1, $2, $3, $4)",
            event_id,
            name,
            id,
            state as UnifiState
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
