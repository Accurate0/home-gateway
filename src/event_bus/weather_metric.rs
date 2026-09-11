use serde::Deserialize;

use super::forecast_day::ForecastDay;
use super::weather_source::WeatherSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WeatherMetric {
    Temperature,
    FeelsLike,
    Humidity,
    WindSpeed,
    GustSpeed,
    MaxGustSpeed,
    #[serde(rename = "rain_since_9am")]
    RainSince9am,
    Uv,
    MaxTemp,
    MinTemp,
    UvMax,
    RainProbability,
    RainMax,
    WindMaxSpeed,
}

impl WeatherMetric {
    pub const BOM: &'static [WeatherMetric] = &[
        WeatherMetric::Temperature,
        WeatherMetric::FeelsLike,
        WeatherMetric::Humidity,
        WeatherMetric::WindSpeed,
        WeatherMetric::GustSpeed,
        WeatherMetric::MaxGustSpeed,
        WeatherMetric::RainSince9am,
        WeatherMetric::Uv,
        WeatherMetric::MaxTemp,
        WeatherMetric::MinTemp,
    ];

    pub const WILLYWEATHER: &'static [WeatherMetric] = &[
        WeatherMetric::MaxTemp,
        WeatherMetric::MinTemp,
        WeatherMetric::UvMax,
        WeatherMetric::RainProbability,
        WeatherMetric::RainMax,
        WeatherMetric::WindMaxSpeed,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            WeatherMetric::Temperature => "temperature",
            WeatherMetric::FeelsLike => "feels_like",
            WeatherMetric::Humidity => "humidity",
            WeatherMetric::WindSpeed => "wind_speed",
            WeatherMetric::GustSpeed => "gust_speed",
            WeatherMetric::MaxGustSpeed => "max_gust_speed",
            WeatherMetric::RainSince9am => "rain_since_9am",
            WeatherMetric::Uv => "uv",
            WeatherMetric::MaxTemp => "max_temp",
            WeatherMetric::MinTemp => "min_temp",
            WeatherMetric::UvMax => "uv_max",
            WeatherMetric::RainProbability => "rain_probability",
            WeatherMetric::RainMax => "rain_max",
            WeatherMetric::WindMaxSpeed => "wind_max_speed",
        }
    }

    pub fn supports(&self, source: WeatherSource) -> bool {
        let metrics = match source {
            WeatherSource::Bom => Self::BOM,
            WeatherSource::WillyWeather => Self::WILLYWEATHER,
        };

        metrics.contains(self)
    }

    pub fn label(&self, day: Option<ForecastDay>) -> String {
        match day {
            Some(day) => format!("{}_{}", day.as_str(), self.as_str()),
            None => self.as_str().to_owned(),
        }
    }

    pub fn validate(&self, source: WeatherSource, day: Option<ForecastDay>) -> Result<(), String> {
        if !self.supports(source) {
            return Err(format!(
                "weather metric `{}` is not provided by source `{}`",
                self.as_str(),
                source.as_str()
            ));
        }

        match (source, day) {
            (WeatherSource::Bom, Some(day)) => Err(format!(
                "weather source `bom` is an observation and does not take `day: {}`",
                day.as_str()
            )),
            (WeatherSource::WillyWeather, None) => Err(format!(
                "weather metric `{}` from `willyweather` requires `day: today|tomorrow`",
                self.as_str()
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_metrics_reject_a_day_and_forecast_metrics_require_one() {
        assert!(
            WeatherMetric::Temperature
                .validate(WeatherSource::Bom, None)
                .is_ok()
        );
        assert!(
            WeatherMetric::Temperature
                .validate(WeatherSource::Bom, Some(ForecastDay::Today))
                .is_err()
        );
        assert!(
            WeatherMetric::MaxTemp
                .validate(WeatherSource::WillyWeather, Some(ForecastDay::Tomorrow))
                .is_ok()
        );
        assert!(
            WeatherMetric::MaxTemp
                .validate(WeatherSource::WillyWeather, None)
                .is_err()
        );
        assert!(
            WeatherMetric::MaxTemp
                .validate(WeatherSource::Bom, None)
                .is_ok()
        );
        assert!(
            WeatherMetric::UvMax
                .validate(WeatherSource::Bom, None)
                .is_err()
        );
        assert!(
            WeatherMetric::RainSince9am
                .validate(WeatherSource::WillyWeather, Some(ForecastDay::Today))
                .is_err()
        );
    }

    #[test]
    fn forecast_rain_and_wind_metrics_are_willyweather_only() {
        for metric in [
            WeatherMetric::RainProbability,
            WeatherMetric::RainMax,
            WeatherMetric::WindMaxSpeed,
        ] {
            assert!(
                metric
                    .validate(WeatherSource::WillyWeather, Some(ForecastDay::Today))
                    .is_ok()
            );
            assert!(metric.validate(WeatherSource::WillyWeather, None).is_err());
            assert!(metric.validate(WeatherSource::Bom, None).is_err());
        }
    }
}
