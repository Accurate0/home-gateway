use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HolidaySettings {
    pub url: String,
    pub regions: Vec<String>,
}
