use async_graphql::{Enum, Object, SimpleObject, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    device_registry::{Capability, DeviceRegistry, EinkDisplaySettings},
    graphql::dataloader::eink_battery::EinkDisplayDataLoader,
    settings::{EinkMode, Orientation},
    timedelta_format::humanize,
};

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
            sleep_start: settings
                .sleep
                .map(|s| s.start.format("%H:%M").to_string()),
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
            .map(crate::battery::voltage_to_percentage))
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
        Ok(registry.eink_display(&self.id).map(EinkDisplayConfig::from))
    }
}
