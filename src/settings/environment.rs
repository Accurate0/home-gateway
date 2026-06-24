use serde::Deserialize;

use super::{DeviceAliases, IEEEAddress, resolve_device};
use crate::esphome::TEMPERATURE_SENSOR_OBJECT_IDS;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EnvironmentSensorType {
    #[default]
    Zigbee,
    Esphome,
}

/// esphome sensor object_ids to subscribe to when none are configured. Matches
/// the readings the environment actor knows how to ingest; boards simply never
/// publish the topics they lack.
fn default_entities() -> Vec<String> {
    TEMPERATURE_SENSOR_OBJECT_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Where an environment sensor's readings come from. The variant is explicit so
/// the map key's meaning (zigbee address vs esphome node name) is never implied.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentSource {
    Zigbee(IEEEAddress),
    Esphome { node: String },
}

impl EnvironmentSource {
    fn identifier(&self) -> String {
        match self {
            EnvironmentSource::Zigbee(addr) => addr.clone(),
            EnvironmentSource::Esphome { node } => node.clone(),
        }
    }

    fn sensor_type(&self) -> EnvironmentSensorType {
        match self {
            EnvironmentSource::Zigbee(_) => EnvironmentSensorType::Zigbee,
            EnvironmentSource::Esphome { .. } => EnvironmentSensorType::Esphome,
        }
    }

    /// Resolve a zigbee address that may be written as a device alias.
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        if let EnvironmentSource::Zigbee(addr) = self {
            *addr = resolve_device(addr, devices)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentSensorSettings {
    pub id: String,
    pub sensor_type: EnvironmentSensorType,
    /// esphome sensor object_ids to subscribe to on discovery. Unused for zigbee
    /// sources (their readings arrive on the `zigbee2mqtt/+` wildcard).
    pub entities: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawEnvironmentSensor {
    id: String,
    source: EnvironmentSource,
    #[serde(default = "default_entities")]
    entities: Vec<String>,
}

impl RawEnvironmentSensor {
    /// Resolve into `(identifier, settings)` for the runtime sensor map.
    pub(super) fn resolve(
        mut self,
        devices: &DeviceAliases,
    ) -> Result<(String, EnvironmentSensorSettings), String> {
        self.source.resolve_devices(devices)?;
        Ok((
            self.source.identifier(),
            EnvironmentSensorSettings {
                id: self.id,
                sensor_type: self.source.sensor_type(),
                entities: self.entities,
            },
        ))
    }
}
