use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use crate::event_bus::WeatherMetric;

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ForecastDetails {
    pub date_time: String,
    pub code: String,
    pub description: String,
    pub emoji: String,
    pub min: i64,
    pub max: i64,
    pub uv: Option<f64>,
    pub rain_probability: Option<i64>,
    pub rain_start_range: Option<i64>,
    pub rain_end_range: Option<i64>,
    pub rain_range_code: Option<String>,
    pub wind_max_speed: Option<f64>,
    pub first_light: Option<String>,
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
    pub last_light: Option<String>,
}

impl ForecastDetails {
    pub fn metric(&self, metric: WeatherMetric) -> Option<f64> {
        match metric {
            WeatherMetric::MaxTemp => Some(self.max as f64),
            WeatherMetric::MinTemp => Some(self.min as f64),
            WeatherMetric::UvMax => self.uv,
            WeatherMetric::RainProbability => self.rain_probability.map(|value| value as f64),
            WeatherMetric::RainMax => self.rain_end_range.map(|value| value as f64),
            WeatherMetric::WindMaxSpeed => self.wind_max_speed,
            _ => None,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ForecastHour {
    pub date_time: String,
    pub temperature: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
    pub wind_direction_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Forecast {
    pub days: Vec<ForecastDetails>,
    pub hours: Vec<ForecastHour>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WillyWeatherForecast {
    pub location: Location,
    pub forecasts: Forecasts,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub region: String,
    pub state: String,
    #[serde(default)]
    pub postcode: Option<String>,
    pub time_zone: String,
    pub lat: f64,
    pub lng: f64,
    pub type_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Forecasts {
    pub weather: Weather,
    pub uv: Uv,
    #[serde(default)]
    pub rainfall: Rainfall,
    #[serde(default)]
    pub wind: Wind,
    #[serde(default)]
    pub temperature: Temperature,
    #[serde(default)]
    pub sunrisesunset: SunriseSunset,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub days: Vec<WeatherDay>,
    pub units: Units,
    pub issue_date_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherDay {
    pub date_time: String,
    pub entries: Vec<WeatherEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherEntry {
    pub date_time: String,
    pub precis_code: String,
    pub precis: String,
    #[serde(default)]
    pub precis_overlay_code: Option<String>,
    pub night: bool,
    pub min: i64,
    pub max: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Units {
    #[serde(default)]
    pub temperature: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uv {
    pub days: Vec<UvDay>,
    pub issue_date_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UvDay {
    pub date_time: String,
    pub entries: Vec<UvEntry>,
    pub alert: Option<Alert>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UvEntry {
    pub date_time: String,
    pub index: f64,
    pub scale: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub max_index: f64,
    pub scale: String,
    pub start_date_time: String,
    pub end_date_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rainfall {
    pub days: Vec<RainfallDay>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RainfallDay {
    pub date_time: String,
    pub entries: Vec<RainfallEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RainfallEntry {
    pub date_time: String,
    pub start_range: Option<i64>,
    pub end_range: Option<i64>,
    pub range_divide: Option<String>,
    pub range_code: Option<String>,
    pub probability: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    pub days: Vec<WindDay>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindDay {
    pub date_time: String,
    pub entries: Vec<WindEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindEntry {
    pub date_time: String,
    pub speed: f64,
    pub direction: Option<f64>,
    pub direction_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Temperature {
    pub days: Vec<TemperatureDay>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemperatureDay {
    pub date_time: String,
    pub entries: Vec<TemperatureEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemperatureEntry {
    pub date_time: String,
    pub temperature: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SunriseSunset {
    pub days: Vec<SunDay>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SunDay {
    pub date_time: String,
    pub entries: Vec<SunEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SunEntry {
    pub first_light_date_time: String,
    pub rise_date_time: String,
    pub set_date_time: String,
    pub last_light_date_time: String,
}
