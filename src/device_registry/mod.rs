use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::RwLock;

use crate::esphome::{EsphomeTarget, motion_state_topic, sensor_state_topic};
use crate::settings::appliance::RawApplianceSettings;
use crate::settings::door::RawDoorSettings;
use crate::settings::environment::default_environment_entities;
use crate::settings::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};
use crate::settings::plant::default_plant_entities;
use crate::settings::{
    ApplianceSettings, DeviceAliases, DoorSettings, EnvironmentSensorSettings,
    EnvironmentSensorType, IEEEAddress, PlantSensorSettings, PresenceSensorType, PresenceSettings,
};
use crate::timedelta_format::option_time_delta_from_str;
use chrono::TimeDelta;

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
    pub kinds: Vec<DeviceConfig>,
    #[serde(default)]
    pub watchdog: Option<RawDeviceWatchdog>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ColourTemp,
    Rgb,
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawDeviceWatchdog {
    #[serde(default, with = "option_time_delta_from_str")]
    timeout: Option<TimeDelta>,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

#[derive(Debug, Clone)]
pub struct DeviceWatchdog {
    pub timeout: Option<TimeDelta>,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", content = "config", rename_all = "snake_case")]
pub enum DeviceConfig {
    Door(RawDoorSettings),
    Appliance(RawApplianceSettings),
    Presence(RawPresenceBlock),
    Environment(RawEnvironmentBlock),
    Plant(RawPlantBlock),
    Light(RawLightBlock),
    ControlSwitch,
    SmartSwitch,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawLightBlock {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPresenceBlock {
    #[serde(default)]
    name: String,
    #[serde(default, deserialize_with = "de_string_or_vec")]
    motion_entity: Vec<String>,
}

fn de_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    Ok(match Option::<OneOrMany>::deserialize(deserializer)? {
        None => Vec::new(),
        Some(OneOrMany::One(entity)) => vec![entity],
        Some(OneOrMany::Many(entities)) => entities,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawEnvironmentBlock {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default = "default_environment_entities")]
    entities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPlantBlock {
    id: String,
    #[serde(default = "default_plant_entities")]
    entities: Vec<String>,
}

#[derive(Debug, Default)]
pub struct DeviceRegistryInner {
    aliases: DeviceAliases,
    esphome_topics: HashMap<String, EsphomeTarget>,
    doors: HashMap<String, DoorSettings>,
    appliances: HashMap<String, ApplianceSettings>,
    environment: HashMap<String, EnvironmentSensorSettings>,
    presence: HashMap<String, PresenceSettings>,
    lights: HashMap<String, String>,
    capabilities: HashMap<String, Vec<Capability>>,
    plant: HashMap<String, PlantSensorSettings>,
    watchdog: HashMap<String, DeviceWatchdog>,
    known_devices: RwLock<HashMap<IEEEAddress, String>>,
}

/// Cheap to clone: the resolved device data lives behind a shared `Arc`, so
/// every actor and the GraphQL schema hold the same registry without each
/// wrapping it in their own `Arc`.
#[derive(Debug, Clone, Default)]
pub struct DeviceRegistry {
    inner: Arc<DeviceRegistryInner>,
}

impl std::ops::Deref for DeviceRegistry {
    type Target = DeviceRegistryInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DeviceRegistry {
    pub fn build(raw: Vec<RawSensor>, notify: &NotifyTargets) -> Result<Self, String> {
        let mut reg = DeviceRegistryInner::default();

        for sensor in raw {
            let RawSensor {
                id,
                transport,
                address,
                kinds,
                watchdog,
                capabilities,
            } = sensor;

            if reg.aliases.insert(id.clone(), address.clone()).is_some() {
                return Err(format!("duplicate sensor id: {id}"));
            }

            if !capabilities.is_empty() {
                reg.capabilities.insert(address.clone(), capabilities);
            }

            if let Some(watchdog) = watchdog {
                reg.watchdog.insert(
                    address.clone(),
                    DeviceWatchdog {
                        timeout: watchdog.timeout,
                        notify: resolve_notify(watchdog.notify, notify)?,
                    },
                );
            }

            for config in kinds {
                reg.add_kind(&id, transport, &address, config, notify)?;
            }
        }

        Ok(Self {
            inner: Arc::new(reg),
        })
    }
}

impl DeviceRegistryInner {
    fn add_kind(
        &mut self,
        id: &str,
        transport: Transport,
        address: &str,
        config: DeviceConfig,
        notify: &NotifyTargets,
    ) -> Result<(), String> {
        match config {
            DeviceConfig::Door(door) => {
                self.doors.insert(address.to_owned(), door.resolve(notify)?);
            }
            DeviceConfig::Appliance(appliance) => {
                self.appliances
                    .insert(address.to_owned(), appliance.resolve(notify)?);
            }
            DeviceConfig::Presence(presence) => {
                if transport == Transport::Esphome {
                    if presence.motion_entity.is_empty() {
                        return Err(format!("esphome presence sensor {id} has no motion_entity"));
                    }
                    for object_id in &presence.motion_entity {
                        self.esphome_topics.insert(
                            motion_state_topic(address, object_id),
                            EsphomeTarget::Motion {
                                node: address.to_owned(),
                                object_id: object_id.clone(),
                            },
                        );
                    }
                }
                self.presence.insert(
                    address.to_owned(),
                    PresenceSettings {
                        name: presence.name,
                        sensor_type: transport.presence_type(),
                        motion_entities: presence.motion_entity,
                    },
                );
            }
            DeviceConfig::Environment(environment) => {
                if transport == Transport::Esphome {
                    self.add_esphome_sensor_topics(address, &environment.entities);
                }
                let name = environment.name.unwrap_or_else(|| environment.id.clone());
                self.environment.insert(
                    address.to_owned(),
                    EnvironmentSensorSettings {
                        id: environment.id,
                        name,
                        sensor_type: transport.environment_type(),
                        entities: environment.entities,
                    },
                );
            }
            DeviceConfig::Plant(plant) => {
                self.add_esphome_sensor_topics(address, &plant.entities);
                self.plant.insert(
                    address.to_owned(),
                    PlantSensorSettings {
                        id: plant.id,
                        entities: plant.entities,
                    },
                );
            }
            DeviceConfig::Light(light) => {
                self.lights.insert(address.to_owned(), light.name);
            }
            DeviceConfig::ControlSwitch | DeviceConfig::SmartSwitch => {}
        }
        Ok(())
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

    pub fn address_or_self<'a>(&'a self, reference: &'a str) -> &'a str {
        self.aliases
            .get(reference)
            .map_or(reference, |a| a.as_str())
    }

    /// Reverse of the alias map: the configured sensor slug for a device address.
    pub fn id_for_address(&self, address: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|(_, a)| a.as_str() == address)
            .map(|(id, _)| id.as_str())
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
                EsphomeTarget::Motion { node, .. } | EsphomeTarget::Sensor { node, .. } => node,
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

    #[allow(unused)]
    pub fn presence(&self, address: &str) -> Option<&PresenceSettings> {
        self.presence.get(address)
    }

    pub fn plant(&self, address: &str) -> Option<&PlantSensorSettings> {
        self.plant.get(address)
    }

    pub fn light(&self, address: &str) -> Option<&String> {
        self.lights.get(address)
    }

    pub fn capabilities(&self, address: &str) -> &[Capability] {
        self.capabilities.get(address).map_or(&[], Vec::as_slice)
    }

    pub fn watchdog_devices(&self) -> impl Iterator<Item = (&String, &DeviceWatchdog)> {
        self.watchdog.iter()
    }

    pub fn lights(&self) -> impl Iterator<Item = (&String, &String)> {
        self.lights.iter()
    }

    /// Every configured door, keyed by address, for entity enumeration.
    pub fn doors(&self) -> impl Iterator<Item = (&String, &DoorSettings)> {
        self.doors.iter()
    }

    /// Every configured presence sensor, keyed by address, for entity enumeration.
    pub fn presence_devices(&self) -> impl Iterator<Item = (&String, &PresenceSettings)> {
        self.presence.iter()
    }

    /// Every configured environment sensor, keyed by address, for entity enumeration.
    pub fn environment_devices(
        &self,
    ) -> impl Iterator<Item = (&String, &EnvironmentSensorSettings)> {
        self.environment.iter()
    }
}
