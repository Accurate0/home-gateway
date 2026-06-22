use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    graphql::dataloader::temperature::{LatestTemperatureDataLoader, TemperatureModel},
    settings::SettingsContainer,
};

pub struct EnvironmentObject {}

pub struct EnvironmentDetails {
    pub id: String,
}

#[Object]
impl EnvironmentObject {
    /// Every environment sensor exposed via config (`config/graphql.yaml`), so
    /// new sensors appear without code changes.
    pub async fn sensors(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<EnvironmentDetails>> {
        let settings = ctx.data::<SettingsContainer>()?.load();
        Ok(settings
            .graphql
            .environments
            .iter()
            .map(|id| EnvironmentDetails { id: id.clone() })
            .collect())
    }

    /// Look up a single environment sensor by its configured id (e.g. `OUTDOOR`).
    pub async fn by_id(
        &self,
        _ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails { id })
    }

    #[graphql(deprecation = "Use `sensors` (config-driven) or `byId` instead")]
    pub async fn outdoor(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails {
            id: "OUTDOOR".to_owned(),
        })
    }

    #[graphql(deprecation = "Use `sensors` (config-driven) or `byId` instead")]
    pub async fn laundry(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails {
            id: "LAUNDRY".to_owned(),
        })
    }

    #[graphql(deprecation = "Use `sensors` (config-driven) or `byId` instead")]
    pub async fn living_room(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails {
            id: "LIVING_ROOM".to_owned(),
        })
    }

    #[graphql(deprecation = "Use `sensors` (config-driven) or `byId` instead")]
    pub async fn bathroom(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails {
            id: "BATHROOM".to_owned(),
        })
    }

    #[graphql(deprecation = "Use `sensors` (config-driven) or `byId` instead")]
    pub async fn bedroom(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentDetails> {
        Ok(EnvironmentDetails {
            id: "BEDROOM".to_owned(),
        })
    }
}

impl EnvironmentDetails {
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
impl EnvironmentDetails {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn temperature(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<f64> {
        self.load(ctx, |t| t.temperature).await
    }

    async fn humidity(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<f64> {
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

    async fn time(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<DateTime<Utc>> {
        self.load(ctx, |t| t.time).await
    }

    async fn name(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<String> {
        self.load(ctx, |t| t.name).await
    }
}
