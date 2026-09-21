use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::RawDeviceWatchdog;
use crate::settings::enabled_state::EnabledState;

use super::device_config::DeviceConfig;
use super::raw_transport::RawTransport;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawDevice {
    pub id: String,
    pub state: EnabledState,
    pub transport: RawTransport,
    #[serde(default)]
    pub model: Option<String>,
    pub roles: Vec<DeviceConfig>,
    #[serde(default)]
    pub watchdog: Option<RawDeviceWatchdog>,
    #[serde(default)]
    pub room: Option<String>,
}
