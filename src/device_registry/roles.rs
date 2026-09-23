use std::collections::BTreeSet;

use crate::decoding::ModelProfile;
use crate::settings::notify::NotifyTargets;
use crate::settings::{
    BatterySettings, DoorSettings, EinkDisplaySettings, EnvironmentSensorSettings,
    MediaPlayerSettings, PlantSensorSettings, PresenceSettings, RobotVacuumSettings, SwitchRole,
    TrmnlDeviceSettings,
};

use super::device_config::DeviceConfig;
use super::transport::Transport;

#[derive(Debug, Clone, Default)]
pub struct Roles {
    pub door: Option<DoorSettings>,
    pub presence: Option<PresenceSettings>,
    pub environment: Option<EnvironmentSensorSettings>,
    pub plant: Option<PlantSensorSettings>,
    pub light: Option<String>,
    pub smart_switch: Option<String>,
    pub control_switch: bool,
    pub battery: Option<BatterySettings>,
    pub eink_display: Option<EinkDisplaySettings>,
    pub trmnl: Option<TrmnlDeviceSettings>,
    pub robot_vacuum: Option<RobotVacuumSettings>,
    pub media_player: Option<MediaPlayerSettings>,
}

pub struct RoleContext<'a> {
    pub id: &'a str,
    pub address: &'a str,
    pub transport: Transport,
    pub profile: Option<&'a ModelProfile>,
    pub notify: &'a NotifyTargets,
}

impl Roles {
    pub fn resolve(cx: &RoleContext, configs: Vec<DeviceConfig>) -> Result<Self, String> {
        let RoleContext { id, transport, .. } = *cx;

        let mut declared = BTreeSet::new();
        let mut roles = Roles::default();

        for config in configs {
            let name = config.role_name();

            if !transport.supports(name) {
                return Err(format!(
                    "device {id}: a `{transport}` device can't declare the `{name}` role"
                ));
            }

            if let Some(profile) = cx.profile
                && !profile.roles.contains(&name)
            {
                return Err(format!(
                    "device {id}: model `{}` has no `{name}` mapping but the device declares a `{name}` role",
                    profile.slug
                ));
            }

            if !declared.insert(name) {
                return Err(format!("device {id}: declares the `{name}` role twice"));
            }

            roles.fill(cx, config)?;
        }

        if let Some(required) = transport.required_role()
            && !declared.contains(&required)
        {
            return Err(format!(
                "device {id}: a `{transport}` device must declare the `{required}` role"
            ));
        }

        Ok(roles)
    }

    fn fill(&mut self, cx: &RoleContext, config: DeviceConfig) -> Result<(), String> {
        let RoleContext {
            id,
            address,
            notify,
            ..
        } = *cx;

        match config {
            DeviceConfig::Door(door) => {
                self.door = Some(door.resolve(notify)?);
            }
            DeviceConfig::Presence(presence) => {
                self.presence = Some(PresenceSettings {
                    name: presence.name,
                });
            }
            DeviceConfig::Environment(environment) => {
                let name = environment.name.unwrap_or_else(|| environment.id.clone());

                self.environment = Some(EnvironmentSensorSettings {
                    id: environment.id,
                    name,
                });
            }
            DeviceConfig::Plant(plant) => {
                self.plant = Some(PlantSensorSettings {
                    id: plant.id,
                    name: plant.name,
                });
            }
            DeviceConfig::Light(light) => {
                self.set_light(id, light.name)?;
            }
            DeviceConfig::ControlSwitch => {
                self.control_switch = true;
            }
            DeviceConfig::SmartSwitch(switch) => {
                if switch.role == Some(SwitchRole::Light) {
                    self.set_light(id, switch.name.clone())?;
                }

                self.smart_switch = Some(switch.name);
            }
            DeviceConfig::EinkDisplayFirmware(display) => {
                self.eink_display = Some(display.resolve(id)?);
            }
            DeviceConfig::Trmnl(trmnl) => {
                self.trmnl = Some(TrmnlDeviceSettings {
                    id: id.to_owned(),
                    name: trmnl.name,
                });
            }
            DeviceConfig::RobotVacuum(robot_vacuum) => {
                let Some(commands) = cx
                    .profile
                    .and_then(|profile| profile.commands.robot_vacuum.clone())
                else {
                    return Err(format!(
                        "device {id}: its model declares no `commands.robot_vacuum`"
                    ));
                };

                self.robot_vacuum = Some(RobotVacuumSettings {
                    name: robot_vacuum.name,
                    commands,
                    address: address.to_owned(),
                });
            }
            DeviceConfig::MediaPlayer(media_player) => {
                if !address.starts_with("media_player.") {
                    return Err(format!(
                        "device {id}: `media_player` address `{address}` must be a home assistant `media_player.` entity id"
                    ));
                }

                self.media_player = Some(media_player.resolve(id, address));
            }
            DeviceConfig::Battery => {
                self.battery = Some(BatterySettings {
                    name: id.to_owned(),
                });
            }
        }

        Ok(())
    }

    fn set_light(&mut self, id: &str, name: String) -> Result<(), String> {
        if self.light.replace(name).is_some() {
            return Err(format!(
                "device {id}: declares both a `light` role and a smart switch `as: light`"
            ));
        }

        Ok(())
    }
}
