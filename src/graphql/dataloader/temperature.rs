use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub struct LatestTemperatureDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct TemperatureModel {
    #[allow(unused)]
    pub id: Uuid,
    pub entity_id: String,
    pub name: String,
    #[allow(unused)]
    pub ieee_addr: String,
    pub temperature: f64,
    #[allow(unused)]
    pub battery: Option<i64>,
    pub humidity: f64,
    pub pressure: Option<f64>,
    #[allow(unused)]
    pub pm25: Option<i64>,
    #[allow(unused)]
    pub voc_index: Option<i64>,
    pub time: DateTime<Utc>,
}

impl Loader<String> for LatestTemperatureDataLoader {
    type Value = TemperatureModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut map = HashMap::new();

        let results = sqlx::query_as!(
            TemperatureModel,
            r#"
            SELECT id, entity_id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, updated_at as time
            FROM latest_temperature_sensor
            WHERE entity_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .await?;

        for result in results {
            map.insert(result.entity_id.clone(), result);
        }

        Ok(map)
    }
}
