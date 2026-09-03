use std::sync::Arc;

use chrono::DateTime;
use moka::future::Cache;
use phf::phf_map;
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::willyweather::types::{Forecast, ForecastDetails, WillyWeatherForecast};
use crate::settings::WillyWeatherSettings;

pub mod types;

const FORECAST_API_TEMPLATE: &str =
    "https://api.willyweather.com.au/v2/{API_KEY}/locations/{LOCATION_ID}/weather.json";
const FORECAST_DAYS: i64 = 7;
const FORECAST_UTC_OFFSET: &str = "+0800";
const CACHE_CAPACITY: u64 = 32;

const PRECIS_TO_EMOJI: phf::Map<&'static str, &'static str> = phf_map! {
    "fine" => "☀️",
    "mostly-fine" => "🌤️",
    "high-cloud" => "☁️",
    "partly-cloudy" => "⛅",
    "mostly-cloudy" => "🌥️",
    "cloudy" => "☁️",
    "overcast" => "🌫️",
    "shower-or-two" => "🌦️",
    "chance-shower-fine" => "🌧️",
    "chance-shower-cloud" => "🌧️",
    "drizzle" => "🌧️",
    "few-showers" => "🌦️",
    "showers-rain" => "🌧️",
    "heavy-showers-rain" => "🌧️",
    "chance-thunderstorm-fine" => "⛈️",
    "chance-thunderstorm-cloud" => "⛈️",
    "chance-thunderstorm-showers" => "⛈️",
    "thunderstorm" => "⛈️",
    "chance-snow-fine" => "🌨️",
    "chance-snow-cloud" => "🌨️",
    "snow-and-rain" => "🌨️",
    "light-snow" => "🌨️",
    "snow" => "❄️",
    "heavy-snow" => "🌨️",
    "wind" => "💨",
    "frost" => "🧊",
    "fog" => "🌁",
    "hail" => "🌨️",
    "dust" => "🌪️",
};

#[derive(thiserror::Error, Debug)]
pub enum WillyWeatherError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("willyweather returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("could not parse forecast timestamp: {0}")]
    Timestamp(#[from] chrono::ParseError),
    #[error("willyweather cache_ttl must be positive")]
    InvalidCacheTtl,
    #[error(transparent)]
    Shared(Arc<WillyWeatherError>),
}

#[derive(Clone)]
pub struct WillyWeather {
    api_key: String,
    client: ClientWithMiddleware,
    cache: Cache<String, Forecast>,
}

impl WillyWeather {
    pub fn new(settings: &WillyWeatherSettings) -> Result<Self, WillyWeatherError> {
        let ttl = settings
            .cache_ttl
            .to_std()
            .map_err(|_| WillyWeatherError::InvalidCacheTtl)?;

        let cache = Cache::builder()
            .max_capacity(CACHE_CAPACITY)
            .time_to_live(ttl)
            .build();

        tracing::info!("willyweather integration enabled (cache ttl {ttl:?})");

        Ok(Self {
            api_key: settings.api_key.clone().unwrap_or_default(),
            client: get_traced_http_client()?,
            cache,
        })
    }

    #[instrument(skip(self))]
    pub async fn forecast(&self, location: &str) -> Result<Forecast, WillyWeatherError> {
        self.cache
            .try_get_with(location.to_owned(), async {
                tracing::debug!("willyweather forecast cache miss for {location}");

                let raw = self.get_forecast(location, FORECAST_DAYS).await?;

                shape_forecast(raw)
            })
            .await
            .map_err(WillyWeatherError::Shared)
    }

    #[instrument(skip(self))]
    pub async fn get_forecast(
        &self,
        location: &str,
        days: i64,
    ) -> Result<WillyWeatherForecast, WillyWeatherError> {
        let url = FORECAST_API_TEMPLATE
            .replace("{API_KEY}", &self.api_key)
            .replace("{LOCATION_ID}", location);

        let response = self
            .client
            .get(url)
            .query(&[("forecasts", "weather,uv"), ("days", &days.to_string())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(WillyWeatherError::Status { status, body });
        }

        Ok(response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }
}

fn shape_forecast(raw: WillyWeatherForecast) -> Result<Forecast, WillyWeatherError> {
    let uv_days = raw.forecasts.uv.days;
    let mut days = Vec::with_capacity(raw.forecasts.weather.days.len());

    for (i, day) in raw.forecasts.weather.days.into_iter().enumerate() {
        let Some(entry) = day.entries.into_iter().next() else {
            tracing::warn!(
                "willyweather day {} has no entries; skipping",
                day.date_time
            );
            continue;
        };

        let datetime = DateTime::parse_from_str(
            &format!("{} {FORECAST_UTC_OFFSET}", entry.date_time),
            "%Y-%m-%d %H:%M:%S %z",
        )?;

        let emoji = PRECIS_TO_EMOJI
            .get(entry.precis_code.as_str())
            .map_or("", |e| e)
            .to_owned();

        let uv = uv_days
            .get(i)
            .and_then(|day| day.alert.as_ref())
            .map(|alert| alert.max_index);

        days.push(ForecastDetails {
            date_time: datetime.to_rfc3339(),
            code: entry.precis_code,
            description: entry.precis,
            emoji,
            min: entry.min,
            max: entry.max,
            uv,
        });
    }

    Ok(Forecast { days })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/willyweather/forecast.json"
    ));

    #[test]
    fn shapes_the_upstream_forecast() {
        let raw: WillyWeatherForecast = serde_json::from_str(FIXTURE).unwrap();
        let forecast = shape_forecast(raw).unwrap();

        assert_eq!(forecast.days.len(), 3);

        let first = &forecast.days[0];
        assert_eq!(first.date_time, "2026-09-03T00:00:00+08:00");
        assert_eq!(first.code, "mostly-fine");
        assert_eq!(first.description, "Mostly sunny");
        assert_eq!(first.emoji, "🌤️");
        assert_eq!(first.min, 9);
        assert_eq!(first.max, 22);
        assert_eq!(first.uv, Some(7.2));

        assert_eq!(forecast.days[1].emoji, "🌧️");
        assert_eq!(forecast.days[1].uv, None, "day without a uv alert");

        assert_eq!(forecast.days[2].emoji, "", "unmapped precis code");
        assert_eq!(forecast.days[2].uv, None, "day beyond the uv forecast");
    }
}
