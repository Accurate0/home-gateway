use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

use crate::integrations::willyweather::{WillyWeather, WillyWeatherError, types::Forecast};

pub struct ForecastDataLoader {
    pub willyweather: WillyWeather,
}

impl Loader<String> for ForecastDataLoader {
    type Value = Forecast;
    type Error = Arc<WillyWeatherError>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut forecasts = HashMap::with_capacity(keys.len());

        for location in keys {
            let forecast = self
                .willyweather
                .forecast(location)
                .await
                .map_err(Arc::new)?;

            forecasts.insert(location.clone(), forecast);
        }

        Ok(forecasts)
    }
}
