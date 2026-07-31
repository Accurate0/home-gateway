use async_graphql::{Object, SimpleObject, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    battery::{BatteryChemistry, voltage_to_percentage},
    graphql::dataloader::{
        device_battery::DeviceBatteryDataLoader,
        device_battery_history::{DeviceBatteryHistoryDataLoader, clamp_since},
    },
};

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct BatteryPoint {
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
    pub time: DateTime<Utc>,
    #[graphql(skip)]
    pub battery_chemistry: Option<BatteryChemistry>,
}

#[async_graphql::ComplexObject]
impl BatteryPoint {
    async fn battery_percentage(&self) -> Option<f64> {
        self.battery_percent.or_else(|| {
            self.battery_voltage
                .and_then(|v| voltage_to_percentage(self.chemistry(), v))
        })
    }
}

impl BatteryPoint {
    fn chemistry(&self) -> BatteryChemistry {
        self.battery_chemistry.unwrap_or(BatteryChemistry::Unknown)
    }
}

pub struct DeviceBattery {
    pub device_id: String,
    pub kind: String,
    pub battery_percent: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub battery_chemistry: Option<BatteryChemistry>,
    pub updated_at: DateTime<Utc>,
}

impl DeviceBattery {
    fn chemistry_or_default(&self) -> BatteryChemistry {
        self.battery_chemistry.unwrap_or(BatteryChemistry::Unknown)
    }
}

#[Object(rename_fields = "camelCase")]
impl DeviceBattery {
    async fn kind(&self) -> &str {
        &self.kind
    }

    async fn voltage(&self) -> Option<f64> {
        self.battery_voltage
    }

    async fn chemistry(&self) -> BatteryChemistry {
        self.chemistry_or_default()
    }

    async fn percentage(&self) -> Option<f64> {
        self.battery_percent.or_else(|| {
            self.battery_voltage
                .and_then(|v| voltage_to_percentage(self.chemistry_or_default(), v))
        })
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<BatteryPoint>> {
        let loader = ctx.data::<DataLoader<DeviceBatteryHistoryDataLoader>>()?;
        let since = clamp_since(since, Utc::now());

        Ok(loader
            .load_one((self.device_id.clone(), since))
            .await?
            .unwrap_or_default()
            .into_iter()
            .map(|p| BatteryPoint {
                battery_voltage: p.battery_voltage,
                battery_percent: p.battery_percent,
                time: p.time,
                battery_chemistry: self.battery_chemistry,
            })
            .collect())
    }
}

pub async fn battery_for(
    ctx: &async_graphql::Context<'_>,
    device_id: &str,
) -> async_graphql::Result<Option<DeviceBattery>> {
    let loader = ctx.data::<DataLoader<DeviceBatteryDataLoader>>()?;

    Ok(loader
        .load_one(device_id.to_owned())
        .await?
        .map(|b| DeviceBattery {
            device_id: b.device_id,
            kind: b.kind,
            battery_percent: b.battery_percent,
            battery_voltage: b.battery_voltage,
            battery_chemistry: b.battery_chemistry,
            updated_at: b.updated_at,
        }))
}
