use super::forecast_day::ForecastDay;
use super::weather_metric::WeatherMetric;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherReading {
    pub metric: WeatherMetric,
    pub day: Option<ForecastDay>,
    pub value: f64,
}
