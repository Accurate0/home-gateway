use async_graphql::{Enum, Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::device_registry::{Capability, DeviceRegistry};
use crate::graphql::dataloader::robot_vacuum_state::{
    RobotVacuumStateDataLoader, RobotVacuumStateModel,
};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RobotVacuumKind {
    Roborock,
    Valetudo,
}

pub struct RobotVacuumEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub capabilities: Vec<Capability>,
    kind: RobotVacuumKind,
}

impl RobotVacuumEntity {
    pub fn from_roborock(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.roborock(address)?;
        let id = registry
            .id_for_address(address)
            .unwrap_or(address)
            .to_owned();

        Some(Self {
            id,
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            capabilities: registry.capabilities(address).to_vec(),
            kind: RobotVacuumKind::Roborock,
        })
    }

    pub fn from_valetudo(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.valetudo(address)?;
        let id = registry
            .id_for_address(address)
            .unwrap_or(address)
            .to_owned();

        Some(Self {
            id,
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            capabilities: registry.capabilities(address).to_vec(),
            kind: RobotVacuumKind::Valetudo,
        })
    }

    async fn model(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<RobotVacuumStateModel>> {
        let loader = ctx.data::<DataLoader<RobotVacuumStateDataLoader>>()?;

        Ok(loader.load_one(self.id.clone()).await?)
    }
}

#[Object]
impl RobotVacuumEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Vacuums
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    async fn kind(&self) -> RobotVacuumKind {
        self.kind
    }

    async fn status(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.state))
    }

    async fn battery_percentage(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self
            .model(ctx)
            .await?
            .and_then(|m| m.battery_level)
            .map(|b| b as f64))
    }

    async fn current_room(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.room))
    }

    async fn fan_speed(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.fan_speed))
    }

    async fn current_clean_area(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self.model(ctx).await?.and_then(|m| m.current_clean_area))
    }

    async fn clean_count(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<i32>> {
        Ok(self.model(ctx).await?.and_then(|m| m.clean_count))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(self.model(ctx).await?.map(|m| m.updated_at))
    }
}
