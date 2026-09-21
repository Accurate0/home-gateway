use schemars::JsonSchema;
use serde::Deserialize;

use crate::decoding::DeviceRoleName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Zigbee,
    Esphome,
    EinkDisplayFirmware,
    Trmnl,
    HomeAssistant,
    Valetudo,
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Transport::Zigbee => "zigbee",
            Transport::Esphome => "esphome",
            Transport::EinkDisplayFirmware => "eink_display_firmware",
            Transport::Trmnl => "trmnl",
            Transport::HomeAssistant => "home_assistant",
            Transport::Valetudo => "valetudo",
        };

        f.write_str(name)
    }
}

impl Transport {
    pub fn roles(self) -> &'static [DeviceRoleName] {
        use DeviceRoleName::*;

        match self {
            Transport::Zigbee => &[
                Battery,
                Door,
                Environment,
                Light,
                SmartSwitch,
                Presence,
                ControlSwitch,
            ],
            Transport::Esphome => &[Environment, Plant, Light, Presence],
            Transport::HomeAssistant => &[
                Battery,
                Door,
                Environment,
                Presence,
                RobotVacuum,
                MediaPlayer,
            ],
            Transport::EinkDisplayFirmware => &[EinkDisplayFirmware, Battery],
            Transport::Trmnl => &[Trmnl, Battery],
            Transport::Valetudo => &[Valetudo, Battery],
        }
    }

    pub fn supports(self, role: DeviceRoleName) -> bool {
        self.roles().contains(&role)
    }

    pub fn required_role(self) -> Option<DeviceRoleName> {
        match self {
            Transport::Zigbee | Transport::Esphome | Transport::HomeAssistant => None,
            Transport::EinkDisplayFirmware => Some(DeviceRoleName::EinkDisplayFirmware),
            Transport::Trmnl => Some(DeviceRoleName::Trmnl),
            Transport::Valetudo => Some(DeviceRoleName::Valetudo),
        }
    }
}
