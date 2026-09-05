use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub struct EnergyConsumptionRow {
    pub id: uuid::Uuid,
    pub energy_used: f64,
    pub solar_exported: f64,
    pub time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EnergyRepo {
    db: Pool<Postgres>,
}

impl EnergyRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record<Tz: chrono::TimeZone>(
        &self,
        energy_used: f64,
        solar_exported: f64,
        time: DateTime<Tz>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO energy_consumption(energy_used, solar_exported, time) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            energy_used,
            solar_exported,
            time
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn history_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<EnergyConsumptionRow>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT * FROM energy_consumption WHERE time >= $1 ORDER BY time ASC",
            since
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| EnergyConsumptionRow {
                id: row.id,
                energy_used: row.energy_used,
                solar_exported: row.solar_exported,
                time: row.time,
            })
            .collect())
    }
}
