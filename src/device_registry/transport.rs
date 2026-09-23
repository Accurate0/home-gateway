use schemars::JsonSchema;
use serde::Deserialize;

use crate::decoding::DeviceRoleName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Mqtt,
    EsphomeNativeApi,
    EinkDisplayFirmware,
    Trmnl,
    HomeAssistant,
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Transport::Mqtt => "mqtt",
            Transport::EsphomeNativeApi => "esphome_native_api",
            Transport::EinkDisplayFirmware => "eink_display_firmware",
            Transport::Trmnl => "trmnl",
            Transport::HomeAssistant => "home_assistant",
        };

        f.write_str(name)
    }
}

impl Transport {
    pub fn supports(self, role: DeviceRoleName) -> bool {
        use DeviceRoleName::*;

        match self {
            Transport::Mqtt => ![EinkDisplayFirmware, Trmnl, MediaPlayer].contains(&role),
            Transport::EsphomeNativeApi => {
                [Battery, Environment, Plant, Light, Presence, MediaPlayer].contains(&role)
            }
            Transport::HomeAssistant => [
                Battery,
                Door,
                Environment,
                Presence,
                RobotVacuum,
                MediaPlayer,
            ]
            .contains(&role),
            Transport::EinkDisplayFirmware => [EinkDisplayFirmware, Battery].contains(&role),
            Transport::Trmnl => [Trmnl, Battery].contains(&role),
        }
    }

    pub fn required_role(self) -> Option<DeviceRoleName> {
        match self {
            Transport::Mqtt | Transport::EsphomeNativeApi | Transport::HomeAssistant => None,
            Transport::EinkDisplayFirmware => Some(DeviceRoleName::EinkDisplayFirmware),
            Transport::Trmnl => Some(DeviceRoleName::Trmnl),
        }
    }
}
