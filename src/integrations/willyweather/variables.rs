use schemars::JsonSchema;

use super::day_variables::WillyweatherDayVariables;
use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables, JsonSchema)]
pub struct WillyweatherVariables {
    pub today: WillyweatherDayVariables,
    pub tomorrow: WillyweatherDayVariables,
}
