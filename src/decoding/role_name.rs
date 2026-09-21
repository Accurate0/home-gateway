use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
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
