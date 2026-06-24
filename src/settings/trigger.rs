//! Declarative event triggers: `{ on: <event>, when?: <condition>, run: <steps> }`.
//!
//! A trigger maps an [`crate::event_bus::EventBusMessage`] to a workflow. The
//! dispatcher matches each event against every trigger's `on`, evaluates the
//! optional `when` gate, and runs `run` on the workflow factory. This replaces
//! the old per-device `actions:` blocks so the event→workflow mapping lives in
//! one place and any producer can drive any workflow.

use chrono::TimeDelta;
use serde::Deserialize;

use super::workflow::{Comparison, Condition, Step};
use super::{DeviceAliases, IEEEAddress, resolve_device, yes};
use crate::actors::cron::schedule::CronSchedule;
use crate::event_bus::SensorMetric;
use crate::timedelta_format::option_time_delta_from_str;

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
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            TriggerMatcher::Door { ieee_addr, .. } | TriggerMatcher::Switch { ieee_addr, .. } => {
                *ieee_addr = resolve_device(ieee_addr, devices)?;
            }
            TriggerMatcher::Presence { sensor, .. }
            | TriggerMatcher::Environment { sensor, .. } => {
                *sensor = resolve_device(sensor, devices)?;
            }
            TriggerMatcher::Cron { .. } => {}
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trigger {
    /// Human-readable name, surfaced in logs and metrics.
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    pub on: TriggerMatcher,
    #[serde(default)]
    pub when: Option<Condition>,
    /// Minimum time between firings of this trigger, e.g. `"24h"`. Tracked in the
    /// `trigger_cooldowns` table (keyed by `name`) so it persists across
    /// restarts — useful for "don't remind me about the plants more than once a
    /// day" style triggers.
    #[serde(default, deserialize_with = "option_time_delta_from_str::deserialize")]
    pub cooldown: Option<TimeDelta>,
    pub run: Vec<Step>,
}

impl Trigger {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        self.on.resolve_devices(devices)?;
        if let Some(when) = &mut self.when {
            when.resolve_devices(devices)?;
        }
        for step in &mut self.run {
            step.resolve_devices(devices)?;
        }
        Ok(())
    }
}
