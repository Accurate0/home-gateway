use chrono::DateTime;
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct PlantReading {
    pub event_id: Uuid,
    pub id: Option<String>,
    pub friendly_name: String,
    pub address: String,
    pub soil_moisture: f64,
}

#[derive(Clone)]
pub struct LatestPlantRow {
    pub id: Uuid,
    pub entity_id: String,
    pub name: String,
    pub address: String,
    pub soil_moisture: f64,
    pub time: DateTime<Utc>,
}

pub struct PlantPoint {
    pub soil_moisture: f64,
    pub time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PlantRepo {
    db: Pool<Postgres>,
}

impl PlantRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.plant.record", err)]
    pub async fn record(&self, reading: &PlantReading) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO plant_sensor (event_id, id, name, address, soil_moisture) VALUES ($1, $2, $3, $4, $5)",
            reading.event_id,
            reading.id,
            reading.friendly_name,
            reading.address,
            reading.soil_moisture,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO latest_plant_sensor (entity_id, name, address, soil_moisture, updated_at) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (entity_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                address = EXCLUDED.address,
                soil_moisture = EXCLUDED.soil_moisture,
                updated_at = EXCLUDED.updated_at
            "#,
            reading.id,
            reading.friendly_name,
            reading.address,
            reading.soil_moisture,
            now,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    #[tracing::instrument(skip_all, name = "db.plant.latest_many", fields(keys = keys.len()), err)]
    pub async fn latest_many(&self, keys: &[String]) -> Result<Vec<LatestPlantRow>, sqlx::Error> {
        sqlx::query_as!(
            LatestPlantRow,
            r#"
            SELECT id, entity_id, name, address, soil_moisture, updated_at as time
            FROM latest_plant_sensor
            WHERE entity_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.plant.history", err)]
    pub async fn history(
        &self,
        entity_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<PlantPoint>, sqlx::Error> {
        sqlx::query_as!(
            PlantPoint,
            r#"
            SELECT soil_moisture, "time"
            FROM plant_sensor
            WHERE id = $1 AND "time" >= $2
            ORDER BY "time"
            "#,
            entity_id,
            since
        )
        .fetch_all(&self.db)
        .await
    }
}
