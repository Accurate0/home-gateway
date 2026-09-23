use async_graphql::{Object, SimpleObject, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    device_registry::DeviceRegistry,
    graphql::{
        dataloader::plant::{LatestPlantDataLoader, PlantModel},
        objects::entity_object::last_seen_for,
    },
    repo::RepoRegistry,
};

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct PlantPoint {
    pub soil_moisture: f64,
    pub time: DateTime<Utc>,
}

pub struct PlantEntity {
    pub id: String,
    pub name: String,
    pub address: String,
    pub room: Option<String>,
}

impl PlantEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.plant(address)?;
        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            address: address.to_owned(),
            room: registry.room(address).map(str::to_owned),
        })
    }

    async fn load<T, F>(
        &self,
        context: &async_graphql::Context<'_>,
        mapping: F,
    ) -> async_graphql::Result<T>
    where
        F: Fn(PlantModel) -> T,
    {
        let loader = context.data::<DataLoader<LatestPlantDataLoader>>()?;
        loader
            .load_one(self.id.clone())
            .await?
            .map(mapping)
            .ok_or(anyhow::Error::msg("no details found for this id").into())
    }
}

#[Object]
impl PlantEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Plants
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

    async fn battery(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<super::DeviceBattery>> {
        super::battery_for(ctx, &self.id).await
    }

    async fn soil_moisture(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(Some(self.load(ctx, |p| p.soil_moisture).await?))
    }

    async fn time(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(Some(self.load(ctx, |p| p.time).await?))
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<PlantPoint>> {
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .plant()
            .history(&self.id, since)
            .await?
            .into_iter()
            .map(|point| PlantPoint {
                soil_moisture: point.soil_moisture,
                time: point.time,
            })
            .collect())
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
