use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::device_registry::Capability;
use crate::graphql::dataloader::valetudo_state::{ValetudoStateDataLoader, ValetudoStateModel};

pub struct ValetudoEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub capabilities: Vec<Capability>,
    pub identifier: String,
}

impl ValetudoEntity {
    async fn model(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<ValetudoStateModel>> {
        let loader = ctx.data::<DataLoader<ValetudoStateDataLoader>>()?;
        Ok(loader.load_one(self.identifier.clone()).await?)
    }
}

#[Object]
impl ValetudoEntity {
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
