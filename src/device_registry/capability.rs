use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::Metric;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, async_graphql::Enum, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Brightness,
    ColourTemp,
    Rgb,
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
    Pm25,
    VocIndex,
}

impl Capability {
    pub fn metric(self) -> Option<Metric> {
        match self {
            Capability::Brightness | Capability::ColourTemp | Capability::Rgb => None,
            Capability::Temperature => Some(Metric::Temperature),
            Capability::Humidity => Some(Metric::Humidity),
            Capability::Pressure => Some(Metric::Pressure),
            Capability::Lux => Some(Metric::Lux),
            Capability::UvIndex => Some(Metric::UvIndex),
            Capability::Pm25 => Some(Metric::Pm25),
            Capability::VocIndex => Some(Metric::VocIndex),
        }
    }
}
