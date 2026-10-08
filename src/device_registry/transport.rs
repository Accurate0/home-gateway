use schemars::JsonSchema;
use serde::Deserialize;

use crate::decoding::DeviceRoleName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, JsonSchema, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Transport {
    Mqtt,
    EsphomeNativeApi,
    EinkDisplayFirmware,
    Trmnl,
    HomeAssistant,
    Tuya,
}

impl Transport {
    pub fn supports(self, role: DeviceRoleName) -> bool {
        use DeviceRoleName::*;

        match self {
            Transport::Mqtt => ![EinkDisplayFirmware, Trmnl, MediaPlayer].contains(&role),
            Transport::EsphomeNativeApi => [
                Battery,
                Environment,
                Plant,
                Light,
                Presence,
                MediaPlayer,
                AirPurifier,
            ]
            .contains(&role),
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
            Transport::Tuya => [GarageDoor, Battery].contains(&role),
        }
    }

    pub fn required_role(self) -> Option<DeviceRoleName> {
        match self {
            Transport::Mqtt
            | Transport::EsphomeNativeApi
            | Transport::HomeAssistant
            | Transport::Tuya => None,
            Transport::EinkDisplayFirmware => Some(DeviceRoleName::EinkDisplayFirmware),
            Transport::Trmnl => Some(DeviceRoleName::Trmnl),
        }
    }
}
