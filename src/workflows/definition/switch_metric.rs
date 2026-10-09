use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwitchMetric {
    Power,
    Voltage,
    Current,
    Energy,
}

impl SwitchMetric {
    pub fn as_str(&self) -> &'static str {
        match self {
            SwitchMetric::Power => "power",
            SwitchMetric::Voltage => "voltage",
            SwitchMetric::Current => "current",
            SwitchMetric::Energy => "energy",
        }
    }
}
