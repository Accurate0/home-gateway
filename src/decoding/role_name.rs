use schemars::JsonSchema;
use serde::Deserialize;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Deserialize,
    JsonSchema,
    strum::Display,
    strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DeviceRoleName {
    Battery,
    Door,
    Environment,
    Plant,
    Light,
    SmartSwitch,
    Presence,
    ControlSwitch,
    RobotVacuum,
    MediaPlayer,
    GarageDoor,
    AirPurifier,
    EinkDisplayFirmware,
    Trmnl,
}

impl DeviceRoleName {
    pub fn has_handler(self) -> bool {
        match self {
            DeviceRoleName::Door
            | DeviceRoleName::Environment
            | DeviceRoleName::Plant
            | DeviceRoleName::Light
            | DeviceRoleName::SmartSwitch
            | DeviceRoleName::Presence
            | DeviceRoleName::ControlSwitch
            | DeviceRoleName::RobotVacuum
            | DeviceRoleName::MediaPlayer
            | DeviceRoleName::GarageDoor
            | DeviceRoleName::AirPurifier => true,
            DeviceRoleName::Battery
            | DeviceRoleName::EinkDisplayFirmware
            | DeviceRoleName::Trmnl => false,
        }
    }
}
