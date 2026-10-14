use async_graphql::{Object, SimpleObject, dataloader::DataLoader};
use chrono::{DateTime, TimeDelta, Utc};

use crate::device_registry::DeviceRegistry;
use crate::graphql::dataloader::air_purifier_state::{
    AirPurifierStateDataLoader, AirPurifierStateModel,
};
use crate::graphql::dataloader::device_metric::{DeviceMetricDataLoader, DeviceMetricKey};
use crate::graphql::objects::device_lock_object::DeviceLockObject;
use crate::graphql::objects::lockable::Lockable;
use crate::history_window::clamp_since;
use crate::repo::RepoRegistry;
use crate::repo::air_purifier::AirPurifierMode;

const AQI_METRIC: &str = "aqi";
const CADR_METRIC: &str = "cadr";
const MAX_HISTORY_WINDOW: TimeDelta = TimeDelta::days(90);

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct AirPurifierPoint {
    pub on: bool,
    pub mode: Option<AirPurifierMode>,
    pub pm25: Option<f64>,
    pub time: DateTime<Utc>,
}

pub struct AirPurifierEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub aliases: Vec<String>,
    address: String,
}

impl AirPurifierEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.air_purifier(address)?;

        Some(Self {
            address: address.to_owned(),
            id: settings.id.clone(),
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            aliases: registry.aliases_for(address).to_vec(),
        })
    }

    async fn model(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<AirPurifierStateModel>> {
        let loader = ctx.data::<DataLoader<AirPurifierStateDataLoader>>()?;

        Ok(loader.load_one(self.id.clone()).await?)
    }

    async fn metric(
        &self,
        ctx: &async_graphql::Context<'_>,
        name: &str,
    ) -> async_graphql::Result<Option<f64>> {
        let loader = ctx.data::<DataLoader<DeviceMetricDataLoader>>()?;

        let key = DeviceMetricKey {
            address: self.address.clone(),
            metric: name.to_owned(),
        };

        Ok(loader.load_one(key).await?.and_then(|row| row.value))
    }
}

impl Lockable for AirPurifierEntity {
    fn device_id(&self) -> &str {
        &self.id
    }
}

#[Object]
impl AirPurifierEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Environment
    }

    pub async fn id(&self) -> &str {
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

    async fn on(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<bool>> {
        Ok(self.model(ctx).await?.map(|m| m.is_on))
    }

    async fn mode(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<AirPurifierMode>> {
        Ok(self.model(ctx).await?.and_then(|m| m.mode))
    }

    async fn speed(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<i32>> {
        Ok(self.model(ctx).await?.and_then(|m| m.speed))
    }

    async fn pm25(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<f64>> {
        Ok(self.model(ctx).await?.and_then(|m| m.pm25))
    }

    async fn filter_life(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self.model(ctx).await?.and_then(|m| m.filter_life))
    }

    async fn aqi(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<f64>> {
        self.metric(ctx, AQI_METRIC).await
    }

    async fn cadr(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<f64>> {
        self.metric(ctx, CADR_METRIC).await
    }

    async fn display(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<bool>> {
        Ok(self.model(ctx).await?.and_then(|m| m.display))
    }

    pub async fn lock(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DeviceLockObject>> {
        self.lock_state(ctx).await
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

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<AirPurifierPoint>> {
        let repos = ctx.data::<RepoRegistry>()?;
        let since = clamp_since(since, Utc::now(), MAX_HISTORY_WINDOW);

        Ok(repos
            .air_purifier()
            .history(&self.id, since)
            .await?
            .into_iter()
            .map(|point| AirPurifierPoint {
                on: point.is_on,
                mode: point.mode,
                pm25: point.pm25,
                time: point.time,
            })
            .collect())
    }
}
