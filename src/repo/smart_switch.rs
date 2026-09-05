use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct SmartSwitchReading {
    pub event_id: Uuid,
    pub friendly_name: String,
    pub ieee_addr: String,
    pub voltage: i64,
    pub power: i64,
    pub current: f64,
    pub energy: f64,
}

#[derive(Clone)]
pub struct SmartSwitchRepo {
    db: Pool<Postgres>,
}

impl SmartSwitchRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn append(&self, reading: &SmartSwitchReading) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO smart_switch (event_id, name, ieee_addr, voltage, power, current_f64, energy) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            reading.event_id,
            reading.friendly_name,
            reading.ieee_addr,
            reading.voltage,
            reading.power,
            reading.current,
            reading.energy,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
