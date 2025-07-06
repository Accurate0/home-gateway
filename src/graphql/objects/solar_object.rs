use async_graphql::{Object, SimpleObject};
use chrono::NaiveDateTime;
use http::Method;
use reqwest::ClientBuilder;
use std::time::Duration;

pub struct SolarObject {}

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
    pub today: Vec<GenerationHistory>,
    pub yesterday: Vec<GenerationHistory>,
}

const SOLAR_API_URL: &str = "https://solar-panels.anurag.sh/api";

#[Object]
impl SolarObject {
    pub async fn current(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<SolarCurrentResponse> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()?;

        let url = format!("{SOLAR_API_URL}/current");
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
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()?;

        let url = format!("{SOLAR_API_URL}/history");
        let mut response = client
            .request(Method::GET, url)
            .send()
            .await?
            .error_for_status()?
            .json::<SolarHistoryResponse>()
            .await?;

        let mut total = Vec::with_capacity(response.yesterday.len() + response.today.len());
        total.append(&mut response.yesterday);
        total.append(&mut response.today);

        total.sort_by_key(|r| r.timestamp);

        Ok(total)
    }
}
