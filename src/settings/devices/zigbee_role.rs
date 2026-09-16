use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZigbeeRoleName {
    Battery,
    Door,
    Environment,
    Light,
    SmartSwitch,
    Presence,
    ControlSwitch,
}

impl std::fmt::Display for ZigbeeRoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ZigbeeRoleName::Battery => "battery",
            ZigbeeRoleName::Door => "door",
            ZigbeeRoleName::Environment => "environment",
            ZigbeeRoleName::Light => "light",
            ZigbeeRoleName::SmartSwitch => "smart_switch",
            ZigbeeRoleName::Presence => "presence",
            ZigbeeRoleName::ControlSwitch => "control_switch",
        };

        f.write_str(name)
    }
}
