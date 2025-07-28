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
    pub event_id: Uuid,
    pub id: String,
    pub name: String,
    #[allow(unused)]
    pub ieee_addr: String,
    pub temperature: f64,
    #[allow(unused)]
    pub battery: Option<i64>,
    pub humidity: f64,
    pub pressure: Option<f64>,
    pub time: DateTime<Utc>,
}

impl Loader<String> for LatestTemperatureDataLoader {
    type Value = TemperatureModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut map = HashMap::new();

        let results = sqlx::query_as!(
            TemperatureModel,
            r#"SELECT event_id, id, name, ieee_addr, temperature, battery, humidity, pressure, time
            FROM (SELECT id as latest_id, max(time)
                FROM temperature_sensor WHERE id = ANY($1) GROUP BY id) as latest_state
            INNER JOIN temperature_sensor ON temperature_sensor.id = latest_state.latest_id
            "#,
            keys
        )
        .fetch_all(&self.database)
        .await?;

        for result in results {
            map.insert(result.id.clone(), result);
        }

        Ok(map)
    }
}
