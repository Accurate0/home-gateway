use crate::battery::voltage_to_percentage;
use async_graphql::{ComplexObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};

pub struct EinkDisplayObject {}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct EinkDisplay {
    pub device_id: String,
    pub name: String,
    pub battery_voltage: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl EinkDisplay {
    async fn battery_percentage(&self) -> Option<f64> {
        self.battery_voltage.map(voltage_to_percentage)
    }
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct BatteryPoint {
    pub battery_voltage: f64,
    pub time: DateTime<Utc>,
}

#[ComplexObject]
impl BatteryPoint {
    async fn battery_percentage(&self) -> f64 {
        voltage_to_percentage(self.battery_voltage)
    }
}

#[Object]
impl EinkDisplayObject {
    pub async fn displays(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<EinkDisplay>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT device_id, name, battery_voltage, updated_at
               FROM eink_display
               ORDER BY device_id ASC"#,
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| EinkDisplay {
            device_id: r.device_id,
            name: r.name,
            battery_voltage: r.battery_voltage,
            updated_at: r.updated_at,
        })
        .collect_vec())
    }

    pub async fn battery_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        device_id: String,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<BatteryPoint>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT battery_voltage, "time"
               FROM device_battery
               WHERE device_id = $1 AND "time" >= $2
               ORDER BY "time" ASC"#,
            device_id,
            since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| BatteryPoint {
            battery_voltage: r.battery_voltage,
            time: r.time,
        })
        .collect_vec())
    }
}
