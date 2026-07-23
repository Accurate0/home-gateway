use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use super::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ArmedDoorStates {
    Armed {
        #[serde(with = "time_delta_from_str")]
        #[schemars(with = "String")]
        timeout: TimeDelta,
    },
    Unarmed,
}

#[derive(Debug, Clone)]
pub struct DoorSettings {
    pub name: String,
    pub id: String,
    pub armed: ArmedDoorStates,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct RawDoorSettings {
    name: String,
    id: String,
    #[serde(flatten)]
    armed: ArmedDoorStates,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

impl RawDoorSettings {
    pub(crate) fn resolve(self, targets: &NotifyTargets) -> Result<DoorSettings, String> {
        Ok(DoorSettings {
            name: self.name,
            id: self.id,
            armed: self.armed,
            notify: resolve_notify(self.notify, targets)?,
        })
    }
}
