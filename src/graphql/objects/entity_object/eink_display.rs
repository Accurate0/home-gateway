use async_graphql::{Enum, Object, SimpleObject, dataloader::DataLoader};
use chrono::{DateTime, Utc};
use itertools::Itertools;

use super::BatteryPoint;
use crate::{
    device_registry::{Capability, DeviceRegistry, EinkDisplaySettings},
    feature_flag::FeatureFlagClient,
    graphql::dataloader::{
        device_battery_history::{DeviceBatteryHistoryDataLoader, clamp_since},
        eink_battery::EinkDisplayDataLoader,
    },
    routes::epd::{EpdConfig, build_epd_config},
    settings::{EinkMode, Orientation, SettingsContainer},
    timedelta_format::humanize,
};
use sqlx::{Pool, Postgres};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EinkDisplayKind {
    Trmnl,
    EinkDisplayFirmware,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EinkDisplayConfig {
    mode: EinkMode,
    view: Option<String>,
    album: Option<String>,
    orientation: Orientation,
    refresh: Option<String>,
    settle: Option<String>,
    sleep_start: Option<String>,
    sleep_end: Option<String>,
}

impl From<&EinkDisplaySettings> for EinkDisplayConfig {
    fn from(settings: &EinkDisplaySettings) -> Self {
        Self {
            mode: settings.mode.name(),
            view: settings.mode.view().map(|v| v.to_owned()),
            album: settings.mode.album().map(|a| a.to_owned()),
            orientation: settings.orientation,
            refresh: settings.refresh.map(humanize),
            settle: settings.settle.map(humanize),
            sleep_start: settings.sleep.map(|s| s.start.format("%H:%M").to_string()),
            sleep_end: settings.sleep.map(|s| s.end.format("%H:%M").to_string()),
        }
    }
}

pub struct EinkDisplayEntity {
    pub id: String,
    pub name: String,
    pub address: String,
    pub kind: EinkDisplayKind,
    pub capabilities: Vec<Capability>,
    pub room: Option<String>,
}

impl EinkDisplayEntity {
    pub fn from_firmware(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.eink_display(address)?;
        let id = registry
            .id_for_address(address)
            .unwrap_or(address)
            .to_owned();
        Some(Self {
            id,
            name: settings.name.clone(),
            address: address.to_owned(),
            kind: EinkDisplayKind::EinkDisplayFirmware,
            capabilities: registry.capabilities(address).to_vec(),
            room: registry.room(address).map(str::to_owned),
        })
    }

    pub fn from_trmnl(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.trmnl_devices().get(address)?;
        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            address: settings.id.clone(),
            kind: EinkDisplayKind::Trmnl,
            capabilities: registry.capabilities(address).to_vec(),
            room: registry.room(address).map(str::to_owned),
        })
    }
}

#[Object]
impl EinkDisplayEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Displays
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn kind(&self) -> EinkDisplayKind {
        self.kind
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn battery_voltage(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let loader = ctx.data::<DataLoader<EinkDisplayDataLoader>>()?;
        Ok(loader
            .load_one(self.address.clone())
            .await?
            .and_then(|d| d.battery_voltage))
    }

    async fn battery_percentage(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let loader = ctx.data::<DataLoader<EinkDisplayDataLoader>>()?;
        Ok(loader
            .load_one(self.address.clone())
            .await?
            .and_then(|d| d.battery_voltage)
            .and_then(|v| {
                crate::battery::voltage_to_percentage(crate::battery::BatteryChemistry::Lipo, v)
            }))
    }

    async fn is_charging(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<bool>> {
        let loader = ctx.data::<DataLoader<EinkDisplayDataLoader>>()?;
        Ok(loader
            .load_one(self.address.clone())
            .await?
            .and_then(|d| d.is_charging))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        let loader = ctx.data::<DataLoader<EinkDisplayDataLoader>>()?;
        Ok(loader
            .load_one(self.address.clone())
            .await?
            .map(|d| d.updated_at))
    }

    async fn config(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<EinkDisplayConfig>> {
        let registry = ctx.data::<DeviceRegistry>()?;
        Ok(registry
            .eink_display(&self.address)
            .map(EinkDisplayConfig::from))
    }

    async fn device_config(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EpdConfig> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let feature_flag_client = ctx.data::<FeatureFlagClient>()?;
        let db = ctx.data::<Pool<Postgres>>()?;
        let settings = ctx.data::<SettingsContainer>()?;
        Ok(build_epd_config(db, feature_flag_client, registry, settings, &self.address, None).await)
    }

    async fn battery(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<super::DeviceBattery>> {
        super::battery_for(ctx, &self.address).await
    }

    async fn battery_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<BatteryPoint>> {
        let loader = ctx.data::<DataLoader<DeviceBatteryHistoryDataLoader>>()?;
        let since = clamp_since(since, Utc::now());

        Ok(loader
            .load_one((self.address.clone(), since))
            .await?
            .unwrap_or_default()
            .into_iter()
            .map(|p| BatteryPoint {
                battery_voltage: p.battery_voltage,
                battery_percent: p.battery_percent,
                time: p.time,
                battery_chemistry: Some(crate::battery::BatteryChemistry::Lipo),
            })
            .collect_vec())
    }
}
