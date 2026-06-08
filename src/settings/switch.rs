use serde::Deserialize;
use std::collections::HashMap;

use super::DeviceAliases;
use super::workflow::ActionSettings;

pub type SwitchActionId = String;

#[derive(Debug, Deserialize, Clone)]
pub struct SwitchSettings {
    #[allow(unused)]
    pub name: String,
    pub actions: HashMap<SwitchActionId, ActionSettings>,
}

impl SwitchSettings {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        for action in self.actions.values_mut() {
            action.resolve_devices(devices)?;
        }

        Ok(())
    }
}
