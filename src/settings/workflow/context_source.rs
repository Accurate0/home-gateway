use schemars::JsonSchema;
use serde::Deserialize;

use crate::integrations::fuelwatch::variables::FuelwatchVariables;
use crate::integrations::willyweather::variables::WillyweatherVariables;
use crate::variables::{Shape, WorkflowContextVariables};

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

    pub fn shape(&self) -> Shape {
        match self {
            ContextSource::Fuelwatch => FuelwatchVariables::shape(),
            ContextSource::Willyweather => WillyweatherVariables::shape(),
        }
    }
}
