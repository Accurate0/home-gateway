use crate::battery::BatteryChemistry;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;
use uuid::Uuid;

pub struct BatteryReading<'a> {
    pub event_id: Uuid,
    pub device_id: &'a str,
    pub name: &'a str,
    pub kind: &'a str,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
    pub battery_chemistry: Option<BatteryChemistry>,
}

#[derive(Clone)]
pub struct BatteryRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct DeviceBatteryRow {
    pub device_id: String,
    pub kind: String,
    pub battery_percent: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub battery_chemistry: Option<BatteryChemistry>,
    pub updated_at: DateTime<Utc>,
}

pub struct BatteryHistoryRow {
    pub device_id: String,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
    pub time: DateTime<Utc>,
}

impl BatteryRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record(&self, reading: BatteryReading<'_>) -> Result<(), sqlx::Error> {
        let BatteryReading {
            event_id,
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry,
        } = reading;

        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO device_battery (event_id, device_id, kind, battery_voltage, battery_percent) VALUES ($1, $2, $3, $4, $5)",
            event_id,
            device_id,
            kind,
            battery_voltage,
            battery_percent,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO device_battery_latest (device_id, name, kind, battery_voltage, battery_percent, battery_chemistry, updated_at) VALUES ($1, $2, $3, $4, $5, $6, now()) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, kind = EXCLUDED.kind, battery_voltage = EXCLUDED.battery_voltage, battery_percent = EXCLUDED.battery_percent, battery_chemistry = COALESCE(EXCLUDED.battery_chemistry, device_battery_latest.battery_chemistry), updated_at = EXCLUDED.updated_at",
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry as Option<BatteryChemistry>,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }
    pub async fn latest_many(&self, keys: &[String]) -> Result<Vec<DeviceBatteryRow>, sqlx::Error> {
        sqlx::query_as!(
            DeviceBatteryRow,
            r#"
            SELECT device_id, kind, battery_percent, battery_voltage, battery_chemistry as "battery_chemistry: BatteryChemistry", updated_at
            FROM device_battery_latest
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-device-battery"))
        .await
    }

    pub async fn history_many(
        &self,
        device_ids: &[String],
        earliest: DateTime<Utc>,
    ) -> Result<Vec<BatteryHistoryRow>, sqlx::Error> {
        sqlx::query_as!(
            BatteryHistoryRow,
            r#"SELECT device_id, battery_voltage, battery_percent, "time"
               FROM device_battery
               WHERE device_id = ANY($1) AND "time" >= $2
               ORDER BY "time" ASC"#,
            device_ids,
            earliest
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-device-battery-history"))
        .await
    }
}
