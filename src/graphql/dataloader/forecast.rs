use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

use crate::integrations::willyweather::types::Forecast;
use crate::repo::WillyWeatherRepo;
use crate::repo::willyweather::WillyWeatherRepoError;

pub struct ForecastDataLoader {
    pub repo: WillyWeatherRepo,
}

impl Loader<String> for ForecastDataLoader {
    type Value = Forecast;
    type Error = Arc<WillyWeatherRepoError>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut forecasts = HashMap::with_capacity(keys.len());

        for location in keys {
            match self.repo.forecast(location).await.map_err(Arc::new)? {
                Some(forecast) => {
                    forecasts.insert(location.clone(), forecast);
                }
                None => {
                    tracing::warn!("no stored willyweather forecast for {location}");
                }
            }
        }

        Ok(forecasts)
    }
}
