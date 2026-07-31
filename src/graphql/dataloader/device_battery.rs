use crate::battery::BatteryChemistry;
use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct DeviceBatteryDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct DeviceBatteryModel {
    pub device_id: String,
    pub kind: String,
    pub battery_percent: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub battery_chemistry: Option<BatteryChemistry>,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for DeviceBatteryDataLoader {
    type Value = DeviceBatteryModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            DeviceBatteryModel,
            r#"
            SELECT device_id, kind, battery_percent, battery_voltage, battery_chemistry as "battery_chemistry: BatteryChemistry", updated_at
            FROM device_battery_latest
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-device-battery"))
        .await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
