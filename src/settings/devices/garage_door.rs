use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::devices::door::ArmedDoorStates;
use crate::settings::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};

#[derive(Debug, Clone)]
pub struct GarageDoorSettings {
    pub name: String,
    pub id: String,
    pub address: String,
    pub armed: ArmedDoorStates,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct RawGarageDoorSettings {
    name: String,
    #[serde(flatten)]
    armed: ArmedDoorStates,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

impl RawGarageDoorSettings {
    pub(crate) fn resolve(
        self,
        id: &str,
        address: &str,
        targets: &NotifyTargets,
    ) -> Result<GarageDoorSettings, String> {
        Ok(GarageDoorSettings {
            name: self.name,
            id: id.to_owned(),
            address: address.to_owned(),
            armed: self.armed,
            notify: resolve_notify(self.notify, targets)?,
        })
    }
}
