use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    Fuelwatch,
}

impl ContextSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContextSource::Fuelwatch => "fuelwatch",
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
        }
    }
}
