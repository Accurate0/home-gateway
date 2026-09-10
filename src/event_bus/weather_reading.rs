use super::forecast_day::ForecastDay;
use super::weather_metric::WeatherMetric;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherReading {
    pub metric: WeatherMetric,
    pub day: Option<ForecastDay>,
    pub value: f64,
}

impl WeatherReading {
    pub fn var_name(&self) -> String {
        self.metric.var_name(self.day)
    }
}
