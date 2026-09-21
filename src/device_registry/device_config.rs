use schemars::JsonSchema;
use serde::Deserialize;

use crate::decoding::DeviceRoleName;
use crate::settings::devices::door::RawDoorSettings;
use crate::settings::{
    RawEinkDisplayBlock, RawEnvironmentBlock, RawLightBlock, RawMediaPlayerBlock, RawPlantBlock,
    RawPresenceBlock, RawRobotVacuumBlock, RawSmartSwitchBlock, RawTrmnlBlock,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum DeviceConfig {
    Door(RawDoorSettings),
    Presence(RawPresenceBlock),
    Environment(RawEnvironmentBlock),
    Plant(RawPlantBlock),
    Light(RawLightBlock),
    ControlSwitch,
    SmartSwitch(RawSmartSwitchBlock),
    EinkDisplayFirmware(RawEinkDisplayBlock),
    Trmnl(RawTrmnlBlock),
    RobotVacuum(RawRobotVacuumBlock),
    MediaPlayer(RawMediaPlayerBlock),
    Battery,
}

impl DeviceConfig {
    pub fn role_name(&self) -> DeviceRoleName {
        match self {
            DeviceConfig::Door(_) => DeviceRoleName::Door,
            DeviceConfig::Presence(_) => DeviceRoleName::Presence,
            DeviceConfig::Environment(_) => DeviceRoleName::Environment,
            DeviceConfig::Plant(_) => DeviceRoleName::Plant,
            DeviceConfig::Light(_) => DeviceRoleName::Light,
            DeviceConfig::ControlSwitch => DeviceRoleName::ControlSwitch,
            DeviceConfig::SmartSwitch(_) => DeviceRoleName::SmartSwitch,
            DeviceConfig::EinkDisplayFirmware(_) => DeviceRoleName::EinkDisplayFirmware,
            DeviceConfig::Trmnl(_) => DeviceRoleName::Trmnl,
            DeviceConfig::RobotVacuum(_) => DeviceRoleName::RobotVacuum,
            DeviceConfig::MediaPlayer(_) => DeviceRoleName::MediaPlayer,
            DeviceConfig::Battery => DeviceRoleName::Battery,
        }
    }
}
