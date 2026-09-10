use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, FixedOffset};
use phf::phf_map;
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::willyweather::types::{
    Forecast, ForecastDetails, ForecastHour, Forecasts, WillyWeatherForecast,
};
use crate::settings::WillyWeatherSettings;

pub mod types;

const FORECAST_API_TEMPLATE: &str =
    "https://api.willyweather.com.au/v2/{API_KEY}/locations/{LOCATION_ID}/weather.json";
const FORECAST_TYPES: &str = "weather,uv,rainfall,wind,temperature,sunrisesunset";
const FORECAST_UTC_OFFSET: &str = "+0800";

pub const FORECAST_OFFSET: FixedOffset = match FixedOffset::east_opt(8 * 3600) {
    Some(offset) => offset,
    None => panic!("invalid willyweather forecast offset"),
};

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
}

#[derive(Clone)]
pub struct WillyWeather {
    api_key: String,
    client: ClientWithMiddleware,
}

pub fn precis_emoji(code: &str) -> String {
    PRECIS_TO_EMOJI.get(code).map_or("", |e| e).to_owned()
}

impl WillyWeather {
    pub fn new(settings: &WillyWeatherSettings) -> Result<Self, WillyWeatherError> {
        tracing::info!(
            "willyweather integration enabled for {} location(s)",
            settings.locations.len()
        );

        Ok(Self {
            api_key: settings.api_key.clone().unwrap_or_default(),
            client: get_traced_http_client()?,
        })
    }

    #[instrument(skip(self))]
    pub async fn fetch(&self, location_id: &str, days: i64) -> Result<Forecast, WillyWeatherError> {
        let raw = self.get_forecast(location_id, days).await?;

        shape_forecast(raw)
    }

    #[instrument(skip(self))]
    pub async fn get_forecast(
        &self,
        location_id: &str,
        days: i64,
    ) -> Result<WillyWeatherForecast, WillyWeatherError> {
        let url = FORECAST_API_TEMPLATE
            .replace("{API_KEY}", &self.api_key)
            .replace("{LOCATION_ID}", location_id);

        let response = self
            .client
            .get(url)
            .with_extension(crate::http::UrlTemplate(
                "/v2/{api_key}/locations/{location_id}/weather.json",
            ))
            .query(&[("forecasts", FORECAST_TYPES), ("days", &days.to_string())])
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

fn local_time(value: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
    DateTime::parse_from_str(
        &format!("{value} {FORECAST_UTC_OFFSET}"),
        "%Y-%m-%d %H:%M:%S %z",
    )
}

fn local_rfc3339(value: &str) -> Result<String, chrono::ParseError> {
    Ok(local_time(value)?.to_rfc3339())
}

fn day_key(date_time: &str) -> String {
    date_time.get(..10).unwrap_or(date_time).to_owned()
}

fn shape_forecast(raw: WillyWeatherForecast) -> Result<Forecast, WillyWeatherError> {
    let Forecasts {
        weather,
        uv,
        rainfall,
        wind,
        temperature,
        sunrisesunset,
    } = raw.forecasts;

    let uv_by_day: HashMap<String, f64> = uv
        .days
        .into_iter()
        .filter_map(|day| {
            day.alert
                .map(|alert| (day_key(&day.date_time), alert.max_index))
        })
        .collect();

    let mut rain_by_day: HashMap<_, _> = rainfall
        .days
        .into_iter()
        .filter_map(|day| {
            let key = day_key(&day.date_time);
            day.entries.into_iter().next().map(|entry| (key, entry))
        })
        .collect();

    let wind_max_by_day: HashMap<String, f64> = wind
        .days
        .iter()
        .filter_map(|day| {
            day.entries
                .iter()
                .map(|entry| entry.speed)
                .reduce(f64::max)
                .map(|max| (day_key(&day.date_time), max))
        })
        .collect();

    let mut sun_by_day: HashMap<_, _> = sunrisesunset
        .days
        .into_iter()
        .filter_map(|day| {
            let key = day_key(&day.date_time);
            day.entries.into_iter().next().map(|entry| (key, entry))
        })
        .collect();

    let mut days = Vec::with_capacity(weather.days.len());

    for day in weather.days {
        let key = day_key(&day.date_time);

        let Some(entry) = day.entries.into_iter().next() else {
            tracing::warn!(
                "willyweather day {} has no entries; skipping",
                day.date_time
            );
            continue;
        };

        let date_time = local_rfc3339(&entry.date_time)?;
        let emoji = precis_emoji(&entry.precis_code);
        let rain = rain_by_day.remove(&key);

        let (first_light, sunrise, sunset, last_light) = match sun_by_day.remove(&key) {
            Some(sun) => (
                Some(local_rfc3339(&sun.first_light_date_time)?),
                Some(local_rfc3339(&sun.rise_date_time)?),
                Some(local_rfc3339(&sun.set_date_time)?),
                Some(local_rfc3339(&sun.last_light_date_time)?),
            ),
            None => (None, None, None, None),
        };

        days.push(ForecastDetails {
            date_time,
            code: entry.precis_code,
            description: entry.precis,
            emoji,
            min: entry.min,
            max: entry.max,
            uv: uv_by_day.get(&key).copied(),
            rain_probability: rain.as_ref().and_then(|rain| rain.probability),
            rain_start_range: rain.as_ref().and_then(|rain| rain.start_range),
            rain_end_range: rain.as_ref().and_then(|rain| rain.end_range),
            rain_range_code: rain.and_then(|rain| rain.range_code),
            wind_max_speed: wind_max_by_day.get(&key).copied(),
            first_light,
            sunrise,
            sunset,
            last_light,
        });
    }

    let mut hours: BTreeMap<String, ForecastHour> = BTreeMap::new();

    for entry in temperature.days.into_iter().flat_map(|day| day.entries) {
        hours.entry(entry.date_time).or_default().temperature = Some(entry.temperature);
    }

    for entry in wind.days.into_iter().flat_map(|day| day.entries) {
        let hour = hours.entry(entry.date_time).or_default();

        hour.wind_speed = Some(entry.speed);
        hour.wind_direction = entry.direction;
        hour.wind_direction_text = entry.direction_text;
    }

    let hours = hours
        .into_iter()
        .map(|(local, hour)| {
            Ok(ForecastHour {
                date_time: local_rfc3339(&local)?,
                ..hour
            })
        })
        .collect::<Result<Vec<_>, chrono::ParseError>>()?;

    Ok(Forecast { days, hours })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::WeatherMetric;

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
        assert_eq!(first.rain_probability, Some(20));
        assert_eq!(first.rain_start_range, None);
        assert_eq!(first.rain_end_range, Some(1));
        assert_eq!(first.wind_max_speed, Some(18.0));
        assert_eq!(
            first.first_light.as_deref(),
            Some("2026-09-03T06:05:00+08:00")
        );
        assert_eq!(first.sunrise.as_deref(), Some("2026-09-03T06:31:00+08:00"));
        assert_eq!(first.sunset.as_deref(), Some("2026-09-03T17:58:00+08:00"));
        assert_eq!(
            first.last_light.as_deref(),
            Some("2026-09-03T18:24:00+08:00")
        );

        let second = &forecast.days[1];
        assert_eq!(second.emoji, "🌧️");
        assert_eq!(second.uv, None, "day without a uv alert");
        assert_eq!(second.rain_range_code.as_deref(), Some("5-10"));
        assert_eq!(second.metric(WeatherMetric::RainMax), Some(10.0));
        assert_eq!(second.metric(WeatherMetric::RainProbability), Some(80.0));
        assert_eq!(second.metric(WeatherMetric::WindMaxSpeed), Some(30.0));
        assert_eq!(second.sunrise, None, "day beyond the sun forecast");

        let third = &forecast.days[2];
        assert_eq!(third.emoji, "", "unmapped precis code");
        assert_eq!(third.uv, None, "day beyond the uv forecast");
        assert_eq!(third.metric(WeatherMetric::RainProbability), None);
        assert_eq!(third.wind_max_speed, None);

        let hours: Vec<_> = forecast
            .hours
            .iter()
            .map(|hour| (hour.date_time.as_str(), hour.temperature, hour.wind_speed))
            .collect();

        assert_eq!(
            hours,
            [
                ("2026-09-03T00:00:00+08:00", Some(12.1), Some(12.5)),
                ("2026-09-03T01:00:00+08:00", Some(11.4), Some(18.0)),
                ("2026-09-03T02:00:00+08:00", Some(10.9), None),
                ("2026-09-04T00:00:00+08:00", None, Some(30.0)),
            ]
        );
        assert_eq!(forecast.hours[1].wind_direction, Some(247.5));
        assert_eq!(
            forecast.hours[1].wind_direction_text.as_deref(),
            Some("WSW")
        );
    }
}
