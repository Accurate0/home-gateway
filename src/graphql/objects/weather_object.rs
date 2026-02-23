use crate::http::get_traced_http_client;
use async_graphql::{Object, SimpleObject};
use http::Method;
use serde::{Deserialize, Serialize};

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
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Forecast> {
        let api_base = std::env::var("BOM_API_BASE").unwrap();
        let client = get_traced_http_client()?;

        let url = format!("{api_base}/forecast");
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
