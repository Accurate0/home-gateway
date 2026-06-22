use serde::Deserialize;

use super::{DeviceAliases, IEEEAddress, resolve_device};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EnvironmentSensorType {
    #[default]
    Zigbee,
    Esphome,
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
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawEnvironmentSensor {
    id: String,
    source: EnvironmentSource,
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
            },
        ))
    }
}
