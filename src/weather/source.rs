use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
pub enum WeatherSource {
    #[serde(rename = "bom")]
    Bom,
    #[serde(rename = "willyweather")]
    WillyWeather,
}

impl WeatherSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeatherSource::Bom => "bom",
            WeatherSource::WillyWeather => "willyweather",
        }
    }
}
