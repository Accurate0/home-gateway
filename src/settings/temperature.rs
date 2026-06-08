use serde::Deserialize;

use super::{DeviceAliases, IEEEAddress, resolve_device};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum TemperatureSensorType {
    #[default]
    Zigbee,
    Esphome,
}

/// Where a temperature sensor's readings come from. The variant is explicit so
/// the map key's meaning (zigbee address vs esphome node name) is never implied.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureSource {
    Zigbee(IEEEAddress),
    Esphome { node: String },
}

impl TemperatureSource {
    fn identifier(&self) -> String {
        match self {
            TemperatureSource::Zigbee(addr) => addr.clone(),
            TemperatureSource::Esphome { node } => node.clone(),
        }
    }

    fn sensor_type(&self) -> TemperatureSensorType {
        match self {
            TemperatureSource::Zigbee(_) => TemperatureSensorType::Zigbee,
            TemperatureSource::Esphome { .. } => TemperatureSensorType::Esphome,
        }
    }

    /// Resolve a zigbee address that may be written as a device alias.
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        if let TemperatureSource::Zigbee(addr) = self {
            *addr = resolve_device(addr, devices)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TemperatureSensorSettings {
    pub id: String,
    pub sensor_type: TemperatureSensorType,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawTemperatureSensor {
    id: String,
    source: TemperatureSource,
}

impl RawTemperatureSensor {
    /// Resolve into `(identifier, settings)` for the runtime sensor map.
    pub(super) fn resolve(
        mut self,
        devices: &DeviceAliases,
    ) -> Result<(String, TemperatureSensorSettings), String> {
        self.source.resolve_devices(devices)?;
        Ok((
            self.source.identifier(),
            TemperatureSensorSettings {
                id: self.id,
                sensor_type: self.source.sensor_type(),
            },
        ))
    }
}
