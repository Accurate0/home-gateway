use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgInterval;
use sqlx::{Pool, Postgres};

pub struct EnergyConsumptionRow {
    pub id: uuid::Uuid,
    pub energy_used: f64,
    pub solar_exported: f64,
    pub time: DateTime<Utc>,
}

pub struct EnergyGapRow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EnergyRepo {
    db: Pool<Postgres>,
}

impl EnergyRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.energy.record", err)]
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

    #[tracing::instrument(skip_all, name = "db.energy.history_since", err)]
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

    #[tracing::instrument(skip_all, name = "db.energy.gaps_since", err)]
    pub async fn gaps_since(
        &self,
        since: DateTime<Utc>,
        interval: chrono::TimeDelta,
    ) -> Result<Vec<EnergyGapRow>, sqlx::Error> {
        let interval = PgInterval::try_from(interval).map_err(sqlx::Error::Encode)?;

        let rows = sqlx::query!(
            r#"
            SELECT previous AS "start!", time AS "end!"
            FROM (
                SELECT time, LAG(time) OVER (ORDER BY time) AS previous
                FROM (
                    SELECT time FROM energy_consumption WHERE time >= $1
                    UNION ALL SELECT $1
                    UNION ALL SELECT now()
                ) readings
            ) steps
            WHERE time - previous > $2
            ORDER BY previous ASC
            "#,
            since,
            interval
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| EnergyGapRow {
                start: row.start,
                end: row.end,
            })
            .collect())
    }
}
