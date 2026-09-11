use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::CacheSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TransperthCacheSettings {
    pub routes: CacheSettings,
    pub timetables: CacheSettings,
}
