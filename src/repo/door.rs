use crate::db::DoorState;
use crate::settings::IEEEAddress;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

pub struct DoorReading {
    pub event_id: Uuid,
    pub friendly_name: String,
    pub ieee_addr: String,
    pub contact: bool,
    pub battery: i64,
}

pub struct DerivedDoorEvent {
    pub event_id: Uuid,
    pub name: String,
    pub id: String,
    pub ieee_addr: String,
    pub state: DoorState,
}

#[derive(Clone)]
pub struct DoorRepo {
    db: Pool<Postgres>,
}

impl DoorRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn append_reading(&self, reading: &DoorReading) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO door_sensor (event_id, name, ieee_addr, contact, battery) VALUES ($1, $2, $3, $4, $5)",
            reading.event_id,
            reading.friendly_name,
            reading.ieee_addr,
            reading.contact,
            reading.battery,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn append_derived(&self, event: &DerivedDoorEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO derived_door_events (event_id, name, id, ieee_addr, state) VALUES ($1, $2, $3, $4, $5)",
            event.event_id,
            event.name,
            event.id,
            event.ieee_addr,
            event.state as DoorState
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn latest_derived_per_door(
        &self,
    ) -> Result<HashMap<IEEEAddress, DoorState>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
        SELECT DISTINCT ON (id) id, name, ieee_addr, state as "state: DoorState"
        FROM derived_door_events
        ORDER BY id, "time" DESC
        "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.ieee_addr, row.state))
            .collect())
    }
}
