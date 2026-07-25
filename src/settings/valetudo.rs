use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawValetudoBlock {
    pub name: String,
    #[serde(default)]
    pub mqtt_prefix: Option<String>,
    #[serde(default)]
    pub start_payload: Option<String>,
    #[serde(default)]
    pub stop_payload: Option<String>,
    #[serde(default)]
    pub dock_payload: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValetudoSettings {
    pub name: String,
    pub command_topic: String,
    pub start_payload: String,
    pub stop_payload: String,
    pub dock_payload: String,
}

impl RawValetudoBlock {
    pub fn resolve(self, address: &str) -> ValetudoSettings {
        let prefix = self
            .mqtt_prefix
            .unwrap_or_else(|| format!("valetudo/{address}"));
        ValetudoSettings {
            name: self.name,
            command_topic: format!("{prefix}/command"),
            start_payload: self.start_payload.unwrap_or_else(|| "start".to_owned()),
            stop_payload: self.stop_payload.unwrap_or_else(|| "stop".to_owned()),
            dock_payload: self
                .dock_payload
                .unwrap_or_else(|| "return_to_base".to_owned()),
        }
    }
}
