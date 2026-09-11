use schemars::JsonSchema;
use serde::Deserialize;

use super::{ActorWorkerSettings, RestartSettings};

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ActorSettings {
    pub restart: RestartSettings,
    pub workers: ActorWorkerSettings,
}
