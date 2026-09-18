use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceRoleName {
    Battery,
    Door,
    Environment,
    Light,
    SmartSwitch,
    Presence,
    ControlSwitch,
    RobotVacuum,
    MediaPlayer,
}

impl DeviceRoleName {
    pub fn zigbee(self) -> bool {
        !matches!(
            self,
            DeviceRoleName::RobotVacuum | DeviceRoleName::MediaPlayer
        )
    }
}

impl std::fmt::Display for DeviceRoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DeviceRoleName::Battery => "battery",
            DeviceRoleName::Door => "door",
            DeviceRoleName::Environment => "environment",
            DeviceRoleName::Light => "light",
            DeviceRoleName::SmartSwitch => "smart_switch",
            DeviceRoleName::Presence => "presence",
            DeviceRoleName::ControlSwitch => "control_switch",
            DeviceRoleName::RobotVacuum => "robot_vacuum",
            DeviceRoleName::MediaPlayer => "media_player",
        };

        f.write_str(name)
    }
}
