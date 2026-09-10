use async_graphql::Object;
use async_graphql::dataloader::DataLoader;

use crate::graphql::dataloader::forecast::ForecastDataLoader;
use crate::integrations::willyweather::types::Forecast;
use crate::settings::SettingsContainer;

pub struct WeatherObject {
    pub location: String,
}

#[Object]
impl WeatherObject {
    pub async fn forecast(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Forecast> {
        let settings = ctx.data::<SettingsContainer>()?;
        let loader = ctx.data::<DataLoader<ForecastDataLoader>>()?;

        let Some(alias) = settings.willyweather.resolve_location(&self.location) else {
            return Err(async_graphql::Error::new(format!(
                "unknown weather location {}",
                self.location
            )));
        };

        loader
            .load_one(alias.to_owned())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| {
                async_graphql::Error::new(format!("no forecast for location {}", self.location))
            })
    }
}
