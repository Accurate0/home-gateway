use serde::Deserialize;

use super::workflow::Comparison;
use super::{DeviceAliases, IEEEAddress, validate_device};
use crate::actors::cron::schedule::CronSchedule;
use crate::event_bus::SensorMetric;

/// Which event a trigger fires on. Mirrors the [`crate::event_bus::EventBusMessage`]
/// variants; the dispatcher matches messages against these.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerMatcher {
    Presence {
        sensor: String,
        present: bool,
    },
    Door {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        open: bool,
    },
    Switch {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        action: String,
    },
    /// Fires on a scalar sensor reading. `metric` is the reading/object_id name
    /// (e.g. `soil_moisture`, `temperature`); the flattened comparison is the
    /// threshold. The dispatcher fires on the rising edge of the comparison.
    Environment {
        sensor: String,
        metric: SensorMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    /// Fires on a recurring schedule. `schedule` is a standard 5-field cron
    /// expression (e.g. `"0 20 * * THU"`), evaluated in local time. Driven by the
    /// [`crate::actors::cron::CronActor`] producer, which matches by trigger name.
    Cron {
        schedule: Box<CronSchedule>,
    },
}

impl TriggerMatcher {
    pub fn describe(&self) -> String {
        match self {
            TriggerMatcher::Presence { sensor, present } => {
                format!("presence({sensor}) -> {present}")
            }
            TriggerMatcher::Door { ieee_addr, open } => {
                format!(
                    "door({ieee_addr}) -> {}",
                    if *open { "open" } else { "closed" }
                )
            }
            TriggerMatcher::Switch { ieee_addr, action } => {
                format!("switch({ieee_addr}) action={action}")
            }
            TriggerMatcher::Environment {
                sensor,
                metric,
                cmp,
            } => {
                format!(
                    "environment({sensor}).{metric:?} {:?} {}",
                    cmp.op, cmp.value
                )
            }
            TriggerMatcher::Cron { schedule } => format!("cron({})", schedule.expression()),
        }
    }

    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            TriggerMatcher::Door { ieee_addr, .. } | TriggerMatcher::Switch { ieee_addr, .. } => {
                validate_device(ieee_addr, devices)?;
            }
            TriggerMatcher::Presence { sensor, .. }
            | TriggerMatcher::Environment { sensor, .. } => {
                validate_device(sensor, devices)?;
            }
            TriggerMatcher::Cron { .. } => {}
        }
        Ok(())
    }
}
