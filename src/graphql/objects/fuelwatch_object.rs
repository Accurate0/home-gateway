use async_graphql::Object;

use crate::integrations::fuelwatch::types::FuelSite;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;

pub struct FuelWatchObject {
    pub postcode: Option<i32>,
}

impl FuelWatchObject {
    async fn cheapest_first(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<FuelSite>> {
        let repos = ctx.data::<RepoRegistry>()?;
        let settings = ctx.data::<SettingsContainer>()?;

        let postcode = self
            .postcode
            .unwrap_or(settings.integrations.fuelwatch.postcode);

        let sites = repos
            .fuelwatch()
            .sites_for_postcode(postcode, limit)
            .await?;

        if sites.is_empty() {
            tracing::warn!("fuelwatch has no stored unleaded prices for postcode {postcode}");
        }

        Ok(sites)
    }
}

#[Object]
impl FuelWatchObject {
    async fn cheapest(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<FuelSite>> {
        Ok(self.cheapest_first(ctx, Some(1)).await?.into_iter().next())
    }

    async fn sites(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<usize>,
    ) -> async_graphql::Result<Vec<FuelSite>> {
        self.cheapest_first(ctx, limit.map(|limit| limit as i64))
            .await
    }
}
