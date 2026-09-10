use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FuelChange {
    Drop,
    TomorrowLower,
    TomorrowHigher,
    Cheapest,
}

impl FuelChange {
    pub fn as_str(&self) -> &'static str {
        match self {
            FuelChange::Drop => "drop",
            FuelChange::TomorrowLower => "tomorrow_lower",
            FuelChange::TomorrowHigher => "tomorrow_higher",
            FuelChange::Cheapest => "cheapest",
        }
    }
}
