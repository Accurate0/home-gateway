use serde::Deserialize;

use super::{DeviceAliases, IEEEAddress, resolve_device};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PresenceSensorType {
    #[default]
    Zigbee,
    Esphome,
}

/// Where presence events come from. The esphome variant carries `motion_entity`
/// so it is impossible to configure an esphome presence sensor without it.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PresenceSource {
    Zigbee(IEEEAddress),
    Esphome { node: String, motion_entity: String },
}

impl PresenceSource {
    fn identifier(&self) -> String {
        match self {
            PresenceSource::Zigbee(addr) => addr.clone(),
            PresenceSource::Esphome { node, .. } => node.clone(),
        }
    }

    fn sensor_type(&self) -> PresenceSensorType {
        match self {
            PresenceSource::Zigbee(_) => PresenceSensorType::Zigbee,
            PresenceSource::Esphome { .. } => PresenceSensorType::Esphome,
        }
    }

    fn motion_entity(&self) -> Option<String> {
        match self {
            PresenceSource::Zigbee(_) => None,
            PresenceSource::Esphome { motion_entity, .. } => Some(motion_entity.clone()),
        }
    }

    /// Resolve a zigbee address that may be written as a device alias.
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        if let PresenceSource::Zigbee(addr) = self {
            *addr = resolve_device(addr, devices)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PresenceSettings {
    #[allow(unused)]
    pub name: String,
    pub sensor_type: PresenceSensorType,
    /// Set when `source` is esphome: the binary_sensor object_id treated as motion.
    pub motion_entity: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawPresenceSettings {
    name: String,
    source: PresenceSource,
}

impl RawPresenceSettings {
    /// Resolve into `(identifier, settings)` for the runtime sensor map. The
    /// event→workflow mapping lives in `triggers:`; this just declares the
    /// sensor so esphome topics can be subscribed and events routed.
    pub(super) fn resolve(
        mut self,
        devices: &DeviceAliases,
    ) -> Result<(String, PresenceSettings), String> {
        self.source.resolve_devices(devices)?;

        Ok((
            self.source.identifier(),
            PresenceSettings {
                name: self.name,
                sensor_type: self.source.sensor_type(),
                motion_entity: self.source.motion_entity(),
            },
        ))
    }
}
