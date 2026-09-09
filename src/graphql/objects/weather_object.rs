use async_graphql::Object;
use async_graphql::dataloader::DataLoader;

use crate::graphql::dataloader::forecast::ForecastDataLoader;
use crate::integrations::willyweather::types::Forecast;

pub struct WeatherObject {
    pub location: String,
}

#[Object]
impl WeatherObject {
    pub async fn forecast(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Forecast> {
        let loader = ctx.data::<DataLoader<ForecastDataLoader>>()?;

        loader
            .load_one(self.location.clone())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| {
                async_graphql::Error::new(format!("no forecast for location {}", self.location))
            })
    }
}
