use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TuyaDeviceSettings {
    pub host: String,
    #[serde(default)]
    pub local_key: Option<String>,
}
