use async_graphql::{Object, SimpleObject};
use http::Method;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

use crate::settings::SettingsContainer;

pub struct WeatherObject {
    pub location: String,
}

#[derive(Serialize, Deserialize, SimpleObject)]
pub struct ForecastDetails {
    pub date_time: String,
    pub code: String,
    pub description: String,
    pub emoji: String,
    pub min: i64,
    pub max: i64,
    pub uv: Option<f64>,
}

#[derive(Serialize, Deserialize, SimpleObject)]
pub struct Forecast {
    pub days: Vec<ForecastDetails>,
}

#[Object]
impl WeatherObject {
    pub async fn forecast(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Forecast> {
        let settings = ctx.data::<SettingsContainer>()?;
        let client = ctx.data::<ClientWithMiddleware>()?;

        let url = format!("{}/forecast", settings.bom.url);
        let response = client
            .request(Method::GET, url)
            .query(&[("location", &self.location)])
            .send()
            .await?
            .error_for_status()?
            .json::<Forecast>()
            .await?;

        Ok(response)
    }
}
