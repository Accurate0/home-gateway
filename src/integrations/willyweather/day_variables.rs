use schemars::JsonSchema;

use super::types::ForecastDetails;
use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables, JsonSchema)]
pub struct WillyweatherDayVariables {
    pub description: String,
    pub emoji: String,
    pub min: i64,
    pub max: i64,
    pub uv: Option<f64>,
    pub rain_probability: Option<i64>,
    pub rain_range: Option<String>,
    pub wind_max_speed: Option<f64>,
    pub sunset: Option<String>,
}

impl From<ForecastDetails> for WillyweatherDayVariables {
    fn from(day: ForecastDetails) -> Self {
        let sunset = day.sunset.as_deref().and_then(|sunset| {
            chrono::DateTime::parse_from_rfc3339(sunset)
                .ok()
                .map(|sunset| sunset.format("%H:%M").to_string())
        });

        WillyweatherDayVariables {
            description: day.description,
            emoji: day.emoji,
            min: day.min,
            max: day.max,
            uv: day.uv,
            rain_probability: day.rain_probability,
            rain_range: day.rain_range_code,
            wind_max_speed: day.wind_max_speed,
            sunset,
        }
    }
}
