use chrono::DateTime;
use chrono::Utc;
use sqlx::{Pool, Postgres};
use tracing::Instrument;
use uuid::Uuid;

pub struct EnvironmentReading {
    pub event_id: Uuid,
    pub id: Option<String>,
    pub friendly_name: String,
    pub ieee_addr: String,
    pub temperature: f64,
    pub battery: Option<i64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub pm25: Option<i64>,
    pub voc_index: Option<i64>,
    pub lux: Option<f64>,
    pub uv_index: Option<f64>,
}

pub struct LatestEnvironmentReading {
    pub temperature: f64,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub lux: Option<f64>,
    pub uv_index: Option<f64>,
}

#[derive(Clone)]
pub struct EnvironmentRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct LatestTemperatureRow {
    pub id: Uuid,
    pub entity_id: String,
    pub name: String,
    pub ieee_addr: String,
    pub temperature: f64,
    pub battery: Option<i64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub pm25: Option<i64>,
    pub voc_index: Option<i64>,
    pub lux: Option<f64>,
    pub uv_index: Option<f64>,
    pub time: DateTime<Utc>,
}

impl EnvironmentRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record(&self, reading: &EnvironmentReading) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, lux, uv_index) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            reading.event_id,
            reading.id,
            reading.friendly_name,
            reading.ieee_addr,
            reading.temperature,
            reading.battery,
            reading.humidity,
            reading.pressure,
            reading.pm25,
            reading.voc_index,
            reading.lux,
            reading.uv_index,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO latest_temperature_sensor (entity_id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, lux, uv_index, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (entity_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                ieee_addr = EXCLUDED.ieee_addr,
                temperature = EXCLUDED.temperature,
                battery = EXCLUDED.battery,
                humidity = EXCLUDED.humidity,
                pressure = EXCLUDED.pressure,
                pm25 = EXCLUDED.pm25,
                voc_index = EXCLUDED.voc_index,
                lux = EXCLUDED.lux,
                uv_index = EXCLUDED.uv_index,
                updated_at = EXCLUDED.updated_at
            "#,
            reading.id,
            reading.friendly_name,
            reading.ieee_addr,
            reading.temperature,
            reading.battery,
            reading.humidity,
            reading.pressure,
            reading.pm25,
            reading.voc_index,
            reading.lux,
            reading.uv_index,
            now,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn latest(
        &self,
        entity_id: &str,
    ) -> Result<Option<LatestEnvironmentReading>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT temperature, humidity, pressure, lux, uv_index \
             FROM latest_temperature_sensor WHERE entity_id = $1",
            entity_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| LatestEnvironmentReading {
            temperature: row.temperature,
            humidity: row.humidity,
            pressure: row.pressure,
            lux: row.lux,
            uv_index: row.uv_index,
        }))
    }
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<LatestTemperatureRow>, sqlx::Error> {
        sqlx::query_as!(
            LatestTemperatureRow,
            r#"
            SELECT id, entity_id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, lux, uv_index, updated_at as time
            FROM latest_temperature_sensor
            WHERE entity_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-temperature"))
        .await
    }
}
