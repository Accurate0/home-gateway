use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::db::GarageDoorState;
use crate::device_registry::DeviceRegistry;
use crate::graphql::dataloader::garage_door_state::{
    GarageDoorStateDataLoader, GarageDoorStateModel,
};

pub struct GarageDoorEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub aliases: Vec<String>,
}

impl GarageDoorEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.garage_door(address)?;

        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            aliases: registry.aliases_for(address).to_vec(),
        })
    }

    async fn model(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GarageDoorStateModel>> {
        let loader = ctx.data::<DataLoader<GarageDoorStateDataLoader>>()?;

        Ok(loader.load_one(self.id.clone()).await?)
    }
}

#[Object]
impl GarageDoorEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Doors
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn aliases(&self) -> &[String] {
        &self.aliases
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn battery(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<super::DeviceBattery>> {
        super::battery_for(ctx, &self.id).await
    }

    async fn state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GarageDoorState>> {
        Ok(self.model(ctx).await?.map(|m| m.state))
    }

    async fn open(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<bool>> {
        Ok(self
            .model(ctx)
            .await?
            .map(|m| m.state != GarageDoorState::Closed))
    }

    async fn contact(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<bool>> {
        Ok(self.model(ctx).await?.and_then(|m| m.contact))
    }

    async fn changed_at(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(self.model(ctx).await?.map(|m| m.changed_at))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(self.model(ctx).await?.map(|m| m.updated_at))
    }
}
