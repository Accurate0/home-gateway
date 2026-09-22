use std::collections::BTreeSet;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::device_registry::{Capability, Transport};
use crate::integrations::mqtt::MqttProtocol;
use crate::lua::{LuaDecoder, LuaError};
use crate::settings::Metric;

use super::model_commands::ModelCommands;
use super::model_entities::ModelEntities;
use super::reading::DeviceReading;
use super::role_name::DeviceRoleName;

#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub transport: Transport,
    pub protocol: Option<MqttProtocol>,
    pub slug: String,
    pub roles: BTreeSet<DeviceRoleName>,
    pub capabilities: Vec<Capability>,
    pub environment: Vec<Metric>,
    pub plant: Vec<String>,
    pub entities: ModelEntities,
    pub commands: ModelCommands,
}

impl ModelProfile {
    pub fn source(&self) -> String {
        match self.protocol {
            Some(protocol) => protocol.to_string(),
            None => self.transport.to_string(),
        }
    }
}

impl ModelProfile {
    pub fn decode<I: Serialize>(
        &self,
        decoder: &LuaDecoder,
        input: &I,
    ) -> Result<DeviceReading, LuaError> {
        decoder.call(&self.slug, "decode", input)
    }

    pub fn encode<I: Serialize, T: DeserializeOwned>(
        &self,
        decoder: &LuaDecoder,
        role: DeviceRoleName,
        input: &I,
    ) -> Result<Option<T>, LuaError> {
        decoder.call_at(&self.slug, &["encode", &role.to_string()], input)
    }
}
