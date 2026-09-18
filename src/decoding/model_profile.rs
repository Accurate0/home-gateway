use std::collections::BTreeSet;

use serde::Serialize;

use crate::lua::{LuaDecoder, LuaError};
use crate::settings::Metric;

use super::reading::DeviceReading;
use super::role_name::DeviceRoleName;

#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub kind: &'static str,
    pub slug: String,
    pub roles: BTreeSet<DeviceRoleName>,
    pub environment: Vec<Metric>,
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
