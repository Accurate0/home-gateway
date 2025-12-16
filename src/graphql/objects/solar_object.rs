use crate::http::get_http_client;
use async_graphql::{Object, SimpleObject};
use chrono::{DateTime, NaiveDateTime, Utc};
use http::Method;

pub struct SolarObject {
    pub since: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentStatisticsAverages {
    pub last_15_mins: Option<f64>,
    pub last_1_hour: Option<f64>,
    pub last_3_hours: Option<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentStatistics {
    pub averages: SolarCurrentStatisticsAverages,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentResponse {
    pub current_production_wh: f64,
    pub today_production_kwh: f64,
    pub yesterday_production_kwh: f64,
    pub month_production_kwh: f64,
    pub all_time_production_kwh: f64,
    pub statistics: SolarCurrentStatistics,
    pub uv_level: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerationHistory {
    pub wh: f64,
    pub at: NaiveDateTime,
    pub uv_level: Option<f64>,
    pub temperature: Option<f64>,
    pub timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SolarHistoryResponse {
    pub history: Vec<GenerationHistory>,
}

const SOLAR_API_URL: &str = "https://solar-panels.anurag.sh/api";

#[Object]
impl SolarObject {
    pub async fn current(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<SolarCurrentResponse> {
        let api_base = std::env::var("SOLAR_API_BASE").unwrap_or_else(|_| SOLAR_API_URL.to_owned());
        let client = get_http_client()?;

        let url = format!("{api_base}/current");
        let response = client
            .request(Method::GET, url)
            .send()
            .await?
            .error_for_status()?
            .json::<SolarCurrentResponse>()
            .await?;

        Ok(response)
    }
    pub async fn history(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<GenerationHistory>> {
        let api_base = std::env::var("SOLAR_API_BASE").unwrap_or_else(|_| SOLAR_API_URL.to_owned());
        let client = get_http_client()?;

        let url = format!("{api_base}/v2/history");
        let response = client
            .request(Method::GET, url)
            .query(&[("since", self.since.naive_utc())])
            .send()
            .await?
            .error_for_status()?
            .json::<SolarHistoryResponse>()
            .await?;

        Ok(response.history)
    }
}
