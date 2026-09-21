use std::sync::Arc;

use crate::decoding::{DecodedDevice, EsphomeEntities, ModelEntities, ModelProfile};

use super::capability::Capability;
use super::roles::Roles;
use super::transport::Transport;

#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub address: String,
    pub transport: Transport,
    pub profile: Option<Arc<ModelProfile>>,
    pub room: Option<String>,
    pub watchdog_key: String,
    pub roles: Roles,
}

impl Device {
    pub fn decoded(&self) -> Option<DecodedDevice> {
        let profile = self.profile.clone()?;

        Some(DecodedDevice {
            id: self.id.clone(),
            address: self.address.clone(),
            profile,
        })
    }

    pub fn capabilities(&self) -> &[Capability] {
        self.profile
            .as_ref()
            .map_or(&[], |profile| profile.capabilities.as_slice())
    }

    pub fn esphome_entities(&self) -> Option<&EsphomeEntities> {
        match &self.profile.as_ref()?.entities {
            ModelEntities::Esphome(entities) => Some(entities),
            ModelEntities::Payload | ModelEntities::HomeAssistant(_) => None,
        }
    }
}
