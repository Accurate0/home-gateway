use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use super::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};
use crate::timedelta_format::option_time_delta_from_str;

#[derive(Debug, Clone)]
pub struct BatterySettings {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawDeviceWatchdog {
    #[serde(default, with = "option_time_delta_from_str")]
    #[schemars(with = "Option<String>")]
    timeout: Option<TimeDelta>,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

impl RawDeviceWatchdog {
    pub(crate) fn resolve(self, targets: &NotifyTargets) -> Result<DeviceWatchdog, String> {
        Ok(DeviceWatchdog {
            timeout: self.timeout,
            notify: resolve_notify(self.notify, targets)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeviceWatchdog {
    pub timeout: Option<TimeDelta>,
    pub notify: Vec<NotifySource>,
}
