use async_graphql::{Enum, Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    device_registry::Capability, graphql::dataloader::eink_battery::EinkDisplayDataLoader,
};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EinkDisplayKind {
    Trmnl,
    EinkDisplayFirmware,
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
}
