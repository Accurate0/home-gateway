use std::collections::BTreeSet;

use serde::Serialize;

use crate::device_registry::{Capability, Transport};
use crate::lua::{LuaDecoder, LuaError};
use crate::settings::Metric;

use super::model_entities::ModelEntities;
use super::reading::DeviceReading;
use super::role_name::DeviceRoleName;

#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub transport: Transport,
    pub slug: String,
    pub roles: BTreeSet<DeviceRoleName>,
    pub capabilities: Vec<Capability>,
    pub environment: Vec<Metric>,
    pub plant: Vec<String>,
    pub entities: ModelEntities,
}

impl ModelProfile {
    pub fn decode<I: Serialize>(
        &self,
        decoder: &LuaDecoder,
        input: &I,
    ) -> Result<DeviceReading, LuaError> {
        decoder.call(&self.slug, "decode", input)
    }
}
