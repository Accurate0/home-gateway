use serde::Deserialize;

use super::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};

#[derive(Debug, Deserialize, Clone)]
pub struct ApplianceCurrentThreshold {
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub struct ApplianceSettings {
    pub name: String,
    pub id: String,
    pub current: ApplianceCurrentThreshold,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RawApplianceSettings {
    name: String,
    id: String,
    current: ApplianceCurrentThreshold,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

impl RawApplianceSettings {
    pub(crate) fn resolve(self, targets: &NotifyTargets) -> Result<ApplianceSettings, String> {
        Ok(ApplianceSettings {
            name: self.name,
            id: self.id,
            current: self.current,
            notify: resolve_notify(self.notify, targets)?,
        })
    }
}
