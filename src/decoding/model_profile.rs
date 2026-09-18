use std::collections::BTreeSet;
use std::sync::Arc;

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
    decoder: Arc<LuaDecoder>,
}

impl ModelProfile {
    pub fn new(
        kind: &'static str,
        slug: String,
        roles: BTreeSet<DeviceRoleName>,
        environment: Vec<Metric>,
        decoder: Arc<LuaDecoder>,
    ) -> Self {
        ModelProfile {
            kind,
            slug,
            roles,
            environment,
            decoder,
        }
    }

    pub fn decode<I: Serialize>(&self, input: &I) -> Result<DeviceReading, LuaError> {
        self.decoder.call(&self.slug, "decode", input)
    }
}
