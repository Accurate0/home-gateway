use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::device_registry::Capability;
use crate::graphql::dataloader::home_assistant_state::HomeAssistantStateDataLoader;

pub struct RoborockEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub capabilities: Vec<Capability>,
    pub status_entity: String,
    pub battery_entity: String,
    pub room_entity: String,
}

impl RoborockEntity {
    async fn state_of(
        &self,
        ctx: &async_graphql::Context<'_>,
        entity_id: &str,
    ) -> async_graphql::Result<Option<String>> {
        let loader = ctx.data::<DataLoader<HomeAssistantStateDataLoader>>()?;
        Ok(loader.load_one(entity_id.to_owned()).await?.map(|s| s.state))
    }
}

#[Object]
impl RoborockEntity {
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
        self.state_of(ctx, &self.status_entity).await
    }

    async fn battery_percentage(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self
            .state_of(ctx, &self.battery_entity)
            .await?
            .and_then(|s| s.parse::<f64>().ok()))
    }

    async fn current_room(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        self.state_of(ctx, &self.room_entity).await
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        let loader = ctx.data::<DataLoader<HomeAssistantStateDataLoader>>()?;
        let entities = [
            self.status_entity.clone(),
            self.battery_entity.clone(),
            self.room_entity.clone(),
        ];

        let mut latest: Option<DateTime<Utc>> = None;
        for entity_id in entities {
            if let Some(model) = loader.load_one(entity_id).await? {
                latest = Some(latest.map_or(model.updated_at, |cur| cur.max(model.updated_at)));
            }
        }

        Ok(latest)
    }
}
