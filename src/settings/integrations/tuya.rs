use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::enabled_state::EnabledState;
use crate::settings::reconnect::ReconnectSettings;
use crate::timedelta_format::time_delta_from_str;

use super::tuya_device::TuyaDeviceSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TuyaSettings {
    pub state: EnabledState,
    pub models: PathBuf,
    pub port: u16,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub heartbeat: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub silence_timeout: TimeDelta,
    pub reconnect: ReconnectSettings,
    pub devices: BTreeMap<String, TuyaDeviceSettings>,
    #[serde(default)]
    pub local_keys: BTreeMap<String, String>,
}

impl TuyaSettings {
    pub fn local_key(&self, id: &str) -> Option<&str> {
        self.local_keys.get(&local_key_name(id)).map(String::as_str)
    }

    pub fn local_key_variable(id: &str) -> String {
        format!(
            "INTEGRATIONS__TUYA__LOCAL_KEYS__{}",
            local_key_name(id).to_uppercase()
        )
    }

    pub fn heartbeat(&self) -> Duration {
        self.heartbeat.to_std().unwrap_or_default()
    }

    pub fn silence_timeout(&self) -> Duration {
        self.silence_timeout.to_std().unwrap_or_default()
    }
}

fn local_key_name(id: &str) -> String {
    id.replace('-', "_").to_lowercase()
}
