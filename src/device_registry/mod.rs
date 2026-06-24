use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::RwLock;

use crate::esphome::{EsphomeTarget, motion_state_topic, sensor_state_topic};
use crate::settings::appliance::RawApplianceSettings;
use crate::settings::door::RawDoorSettings;
use crate::settings::environment::default_environment_entities;
use crate::settings::notify::NotifyTargets;
use crate::settings::plant::default_plant_entities;
use crate::settings::{
    ApplianceSettings, DeviceAliases, DoorSettings, EnvironmentSensorSettings,
    EnvironmentSensorType, IEEEAddress, PlantSensorSettings, PresenceSensorType, PresenceSettings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Zigbee,
    Esphome,
}

impl Transport {
    fn environment_type(self) -> EnvironmentSensorType {
        match self {
            Transport::Zigbee => EnvironmentSensorType::Zigbee,
            Transport::Esphome => EnvironmentSensorType::Esphome,
        }
    }

    fn presence_type(self) -> PresenceSensorType {
        match self {
            Transport::Zigbee => PresenceSensorType::Zigbee,
            Transport::Esphome => PresenceSensorType::Esphome,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSensor {
    pub id: String,
    pub transport: Transport,
    pub address: String,
    #[serde(flatten)]
    config: DeviceConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", content = "config", rename_all = "snake_case")]
enum DeviceConfig {
    Door(RawDoorSettings),
    Appliance(RawApplianceSettings),
    Presence(RawPresenceBlock),
    Environment(RawEnvironmentBlock),
    Plant(RawPlantBlock),
    Light,
    ControlSwitch,
    SmartSwitch,
}

#[derive(Debug, Clone, Deserialize)]
struct RawPresenceBlock {
    #[serde(default)]
    name: String,
    #[serde(default)]
    motion_entity: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawEnvironmentBlock {
    id: String,
    #[serde(default = "default_environment_entities")]
    entities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawPlantBlock {
    id: String,
    #[serde(default = "default_plant_entities")]
    entities: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DeviceRegistry {
    aliases: DeviceAliases,
    esphome_topics: HashMap<String, EsphomeTarget>,
    doors: HashMap<String, DoorSettings>,
    appliances: HashMap<String, ApplianceSettings>,
    environment: HashMap<String, EnvironmentSensorSettings>,
    presence: HashMap<String, PresenceSettings>,
    plant: HashMap<String, PlantSensorSettings>,
    known_devices: Arc<RwLock<HashMap<IEEEAddress, String>>>,
}

impl DeviceRegistry {
    pub fn build(raw: Vec<RawSensor>, notify: &NotifyTargets) -> Result<Self, String> {
        let mut reg = DeviceRegistry::default();

        for sensor in raw {
            let RawSensor {
                id,
                transport,
                address,
                config,
            } = sensor;

            if reg.aliases.insert(id.clone(), address.clone()).is_some() {
                return Err(format!("duplicate sensor id: {id}"));
            }

            match config {
                DeviceConfig::Door(door) => {
                    reg.doors.insert(address.clone(), door.resolve(notify)?);
                }
                DeviceConfig::Appliance(appliance) => {
                    reg.appliances
                        .insert(address.clone(), appliance.resolve(notify)?);
                }
                DeviceConfig::Presence(presence) => {
                    if transport == Transport::Esphome {
                        if let Some(motion_entity) = &presence.motion_entity {
                            reg.esphome_topics.insert(
                                motion_state_topic(&address, motion_entity),
                                EsphomeTarget::Motion {
                                    node: address.clone(),
                                },
                            );
                        } else {
                            return Err(format!(
                                "esphome presence sensor {id} has no motion_entity"
                            ));
                        }
                    }
                    reg.presence.insert(
                        address.clone(),
                        PresenceSettings {
                            name: presence.name,
                            sensor_type: transport.presence_type(),
                            motion_entity: presence.motion_entity,
                        },
                    );
                }
                DeviceConfig::Environment(environment) => {
                    if transport == Transport::Esphome {
                        reg.add_esphome_sensor_topics(&address, &environment.entities);
                    }
                    reg.environment.insert(
                        address.clone(),
                        EnvironmentSensorSettings {
                            id: environment.id,
                            sensor_type: transport.environment_type(),
                            entities: environment.entities,
                        },
                    );
                }
                DeviceConfig::Plant(plant) => {
                    reg.add_esphome_sensor_topics(&address, &plant.entities);
                    reg.plant.insert(
                        address.clone(),
                        PlantSensorSettings {
                            id: plant.id,
                            entities: plant.entities,
                        },
                    );
                }
                DeviceConfig::Light | DeviceConfig::ControlSwitch | DeviceConfig::SmartSwitch => {}
            }
        }

        Ok(reg)
    }

    fn add_esphome_sensor_topics(&mut self, node: &str, entities: &[String]) {
        for object_id in entities {
            self.esphome_topics.insert(
                sensor_state_topic(node, object_id),
                EsphomeTarget::Sensor {
                    node: node.to_string(),
                    object_id: object_id.clone(),
                },
            );
        }
    }

    pub fn aliases(&self) -> &DeviceAliases {
        &self.aliases
    }

    pub async fn record_friendly_name(&self, address: IEEEAddress, name: String) {
        self.known_devices.write().await.insert(address, name);
    }

    pub async fn friendly_name(&self, address: &str) -> Option<String> {
        self.known_devices.read().await.get(address).cloned()
    }

    pub fn esphome_target(&self, topic: &str) -> Option<&EsphomeTarget> {
        self.esphome_topics.get(topic)
    }

    pub fn esphome_topics_for<'a>(
        &'a self,
        node: &'a str,
    ) -> impl Iterator<Item = (&'a String, &'a EsphomeTarget)> {
        self.esphome_topics.iter().filter(move |(_, target)| {
            let target_node = match target {
                EsphomeTarget::Motion { node } | EsphomeTarget::Sensor { node, .. } => node,
            };
            target_node == node
        })
    }

    pub fn door(&self, address: &str) -> Option<&DoorSettings> {
        self.doors.get(address)
    }

    pub fn appliance(&self, address: &str) -> Option<&ApplianceSettings> {
        self.appliances.get(address)
    }

    pub fn environment(&self, address: &str) -> Option<&EnvironmentSensorSettings> {
        self.environment.get(address)
    }

    pub fn presence(&self, address: &str) -> Option<&PresenceSettings> {
        self.presence.get(address)
    }

    pub fn plant(&self, address: &str) -> Option<&PlantSensorSettings> {
        self.plant.get(address)
    }
}
