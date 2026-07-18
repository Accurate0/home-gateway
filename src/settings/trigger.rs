use serde::Deserialize;

use super::workflow::Comparison;
use super::{DeviceAliases, IEEEAddress, validate_device};
use crate::actors::cron::schedule::CronSchedule;
use crate::actors::sun::calc::SunTransition;
use crate::event_bus::SensorMetric;
use crate::mode::Mode;

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
    /// Fires at a sun transition (`sunrise`/`sunset`), driven by the
    /// [`crate::actors::sun::SunActor`] producer.
    Sun {
        transition: SunTransition,
        #[serde(
            default,
            deserialize_with = "crate::timedelta_format::signed_time_delta_from_str::deserialize"
        )]
        offset: chrono::TimeDelta,
    },
    /// Fires when a house mode is entered (`active: true`) or exited
    /// (`active: false`), driven by `set_mode`.
    Mode {
        mode: Mode,
        active: bool,
    },
    /// Fires when a Home Assistant entity changes state, driven by the
    /// [`crate::actors::home_assistant`] producer. Optionally gate on the entity
    /// reaching a specific `state`.
    HomeAssistant {
        entity_id: String,
        #[serde(default)]
        state: Option<String>,
    },
}

impl TriggerMatcher {
    // used by the workflow `plan` renderer, currently exercised only in tests
    #[allow(dead_code)]
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
            TriggerMatcher::Mode { mode, active } => {
                format!("mode({}) -> {active}", mode.as_str())
            }
            TriggerMatcher::HomeAssistant { entity_id, state } => match state {
                Some(state) => format!("home_assistant({entity_id}) -> {state}"),
                None => format!("home_assistant({entity_id})"),
            },
            TriggerMatcher::Cron { schedule } => format!("cron({})", schedule.expression()),
            TriggerMatcher::Sun { transition, offset } => {
                if offset.is_zero() {
                    format!("sun -> {transition:?}")
                } else {
                    format!(
                        "sun -> {transition:?} (offset {})",
                        crate::timedelta_format::humanize(*offset)
                    )
                }
            }
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
            TriggerMatcher::Cron { .. }
            | TriggerMatcher::Sun { .. }
            | TriggerMatcher::Mode { .. }
            | TriggerMatcher::HomeAssistant { .. } => {}
        }
        Ok(())
    }
}
