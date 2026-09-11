use schemars::JsonSchema;
use serde::Deserialize;

const PERTH_LATITUDE: f64 = -31.952429;
const PERTH_LONGITUDE: f64 = 115.842283;

fn default_latitude() -> f64 {
    PERTH_LATITUDE
}

fn default_longitude() -> f64 {
    PERTH_LONGITUDE
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct LocationSettings {
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
}

impl Default for LocationSettings {
    fn default() -> Self {
        Self {
            latitude: PERTH_LATITUDE,
            longitude: PERTH_LONGITUDE,
        }
    }
}
