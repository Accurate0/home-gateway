use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    EinkDisplayFirmware,
    Trmnl,
}

impl DeviceRoleName {
    pub const ALL: [DeviceRoleName; 12] = [
        DeviceRoleName::Battery,
        DeviceRoleName::Door,
        DeviceRoleName::Environment,
        DeviceRoleName::Plant,
        DeviceRoleName::Light,
        DeviceRoleName::SmartSwitch,
        DeviceRoleName::Presence,
        DeviceRoleName::ControlSwitch,
        DeviceRoleName::RobotVacuum,
        DeviceRoleName::MediaPlayer,
        DeviceRoleName::EinkDisplayFirmware,
        DeviceRoleName::Trmnl,
    ];

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
            | DeviceRoleName::MediaPlayer => true,
            DeviceRoleName::Battery
            | DeviceRoleName::EinkDisplayFirmware
            | DeviceRoleName::Trmnl => false,
        }
    }
}

impl std::fmt::Display for DeviceRoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DeviceRoleName::Battery => "battery",
            DeviceRoleName::Door => "door",
            DeviceRoleName::Environment => "environment",
            DeviceRoleName::Plant => "plant",
            DeviceRoleName::Light => "light",
            DeviceRoleName::SmartSwitch => "smart_switch",
            DeviceRoleName::Presence => "presence",
            DeviceRoleName::ControlSwitch => "control_switch",
            DeviceRoleName::RobotVacuum => "robot_vacuum",
            DeviceRoleName::MediaPlayer => "media_player",
            DeviceRoleName::EinkDisplayFirmware => "eink_display_firmware",
            DeviceRoleName::Trmnl => "trmnl",
        };

        f.write_str(name)
    }
}
