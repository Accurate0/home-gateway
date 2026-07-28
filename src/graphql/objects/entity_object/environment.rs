use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    device_registry::{Capability, DeviceRegistry},
    graphql::{
        dataloader::temperature::{LatestTemperatureDataLoader, TemperatureModel},
        objects::entity_object::last_seen_for,
    },
};

pub struct EnvironmentEntity {
    /// configured entity id (e.g. `outdoor`), the dataloader key.
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// device address (ieee addr or esphome node), the last-seen dataloader key.
    pub address: String,
    pub capabilities: Vec<Capability>,
    pub room: Option<String>,
}

impl EnvironmentEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.environment(address)?;
        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            address: address.to_owned(),
            capabilities: registry.capabilities(address).to_vec(),
            room: registry.room(address).map(str::to_owned),
        })
    }

    async fn load<T, F>(
        &self,
        context: &async_graphql::Context<'_>,
        mapping: F,
    ) -> async_graphql::Result<T>
    where
        F: Fn(TemperatureModel) -> T,
    {
        let loader = context.data::<DataLoader<LatestTemperatureDataLoader>>()?;
        loader
            .load_one(self.id.clone())
            .await?
            .map(mapping)
            .ok_or(anyhow::Error::msg("no details found for this id").into())
    }
}

#[Object]
impl EnvironmentEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Environment
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    /// Nullable so a missing reading reports the error against this field
    /// without nulling the whole entity.
    async fn temperature(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(Some(self.load(ctx, |t| t.temperature).await?))
    }

    async fn humidity(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.humidity).await
    }

    async fn pressure(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.pressure).await
    }

    async fn lux(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.lux).await
    }

    async fn uv_index(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.uv_index).await
    }

    /// Nullable so a missing reading reports the error against this field
    /// without nulling the whole entity.
    async fn time(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(Some(self.load(ctx, |t| t.time).await?))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
