use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    Fuelwatch,
    Willyweather,
}

impl ContextSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContextSource::Fuelwatch => "fuelwatch",
            ContextSource::Willyweather => "willyweather",
        }
    }

    pub fn available_vars(&self) -> &'static [&'static str] {
        match self {
            ContextSource::Fuelwatch => &[
                "fuel_price",
                "fuel_brand",
                "fuel_name",
                "fuel_suburb",
                "fuel_address",
            ],
            ContextSource::Willyweather => &[
                "forecast_description",
                "forecast_emoji",
                "forecast_min",
                "forecast_max",
                "forecast_uv",
                "forecast_rain_probability",
                "forecast_rain_range",
                "forecast_wind_max_speed",
                "forecast_sunset",
            ],
        }
    }
}
