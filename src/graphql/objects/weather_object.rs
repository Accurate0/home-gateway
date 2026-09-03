use async_graphql::Object;

use crate::integrations::willyweather::WillyWeather;
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
        let willyweather = ctx.data::<WillyWeather>()?;

        Ok(willyweather.forecast(&self.location).await?)
    }
}
