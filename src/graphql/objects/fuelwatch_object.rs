use async_graphql::Object;

use crate::integrations::fuelwatch::FuelWatch;
use crate::integrations::fuelwatch::types::FuelSite;

pub struct FuelWatchObject {
    pub postcode: Option<i32>,
}

#[Object]
impl FuelWatchObject {
    async fn cheapest(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<FuelSite>> {
        let Some(fuelwatch) = ctx.data::<Option<FuelWatch>>()? else {
            return Ok(None);
        };

        Ok(fuelwatch.sites(self.postcode).await?.into_iter().next())
    }

    async fn sites(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<usize>,
    ) -> async_graphql::Result<Vec<FuelSite>> {
        let Some(fuelwatch) = ctx.data::<Option<FuelWatch>>()? else {
            return Ok(Vec::new());
        };

        let mut sites = fuelwatch.sites(self.postcode).await?;

        if let Some(limit) = limit {
            sites.truncate(limit);
        }

        Ok(sites)
    }
}
