use crate::settings::NotifySource;
use crate::settings::trigger::TriggerMatcher;
use crate::timedelta_format::option_time_delta_from_str;

use super::{DeviceAliases, IEEEAddress, validate_device, yes};
use crate::actors::sun::calc::SunPeriod;
use chrono::{NaiveTime, TimeDelta};
use serde::Deserialize;

/// Brightness / colour-temperature mutations applied to a light. Kept in
/// `SCREAMING_SNAKE_CASE` to match the long-standing on-disk config.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LightState {
    On,
    Off,
    Toggle,
    SetBrightness {
        value: u64,
    },
    IncreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    DecreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    IncreaseColourTemperature {
        value: u64,
    },
    DecreaseColourTemperature {
        value: u64,
    },
    StopColourTemperature,
    StopBrightness,
}

/// On/off set command for a smart switch / plug.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwitchState {
    On,
    Off,
    Toggle,
}

/// Which reading of an environment sensor a condition compares against.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnvMetric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
}

/// A scalar comparison: `{ op: gt, value: 30 }`. Flattened into the
/// [`Condition::Environment`] variant.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Comparison {
    pub op: CompareOp,
    pub value: f64,
}

impl Comparison {
    pub fn matches(&self, actual: f64) -> bool {
        match self.op {
            CompareOp::Gt => actual > self.value,
            CompareOp::Lt => actual < self.value,
            CompareOp::Gte => actual >= self.value,
            CompareOp::Lte => actual <= self.value,
            // direct float equality is intentional: thresholds are configured as
            // exact values and sensors report discrete readings
            CompareOp::Eq => actual == self.value,
        }
    }
}

/// A boolean predicate evaluated against current device/sensor state. Recursive
/// via `all`/`any`/`not` so arbitrary boolean expressions are expressible.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    Light {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        on: bool,
    },
    Environment {
        sensor: String,
        metric: EnvMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    Presence {
        sensor: String,
        present: bool,
    },
    Door {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        open: bool,
    },
    TimeOfDay {
        #[serde(default)]
        after: Option<NaiveTime>,
        #[serde(default)]
        before: Option<NaiveTime>,
    },
    Sun {
        is: SunPeriod,
        #[serde(
            default,
            deserialize_with = "crate::timedelta_format::signed_time_delta_from_str::deserialize"
        )]
        offset: TimeDelta,
    },
    All {
        conditions: Vec<Condition>,
    },
    Any {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
}

impl Condition {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Condition::Light { ieee_addr, .. } | Condition::Door { ieee_addr, .. } => {
                validate_device(ieee_addr, devices)?;
            }
            Condition::All { conditions } | Condition::Any { conditions } => {
                for c in conditions {
                    c.resolve_devices(devices)?;
                }
            }
            Condition::Not { condition } => condition.resolve_devices(devices)?,
            Condition::Environment { .. }
            | Condition::Presence { .. }
            | Condition::TimeOfDay { .. }
            | Condition::Sun { .. } => {}
        }
        Ok(())
    }

    pub fn describe(&self) -> String {
        match self {
            Condition::Light { ieee_addr, on } => {
                format!("light({ieee_addr}) is {}", if *on { "on" } else { "off" })
            }
            Condition::Environment {
                sensor,
                metric,
                cmp,
            } => format!("env({sensor}).{metric:?} {:?} {}", cmp.op, cmp.value),
            Condition::Presence { sensor, present } => {
                format!("presence({sensor}) is {present}")
            }
            Condition::Door { ieee_addr, open } => {
                format!(
                    "door({ieee_addr}) is {}",
                    if *open { "open" } else { "closed" }
                )
            }
            Condition::TimeOfDay { after, before } => match (after, before) {
                (Some(a), Some(b)) => format!("time in [{a}, {b})"),
                (Some(a), None) => format!("time after {a}"),
                (None, Some(b)) => format!("time before {b}"),
                (None, None) => "time always".to_string(),
            },
            Condition::Sun { is, offset } => {
                if offset.is_zero() {
                    format!("sun is {is:?}")
                } else {
                    format!(
                        "sun is {is:?} (offset {})",
                        crate::timedelta_format::humanize(*offset)
                    )
                }
            }
            Condition::All { conditions } => {
                format!("all[{}]", describe_join(conditions))
            }
            Condition::Any { conditions } => {
                format!("any[{}]", describe_join(conditions))
            }
            Condition::Not { condition } => format!("not({})", condition.describe()),
        }
    }
}

fn describe_join(conditions: &[Condition]) -> String {
    conditions
        .iter()
        .map(Condition::describe)
        .collect::<Vec<_>>()
        .join(", ")
}

/// A single workflow step: one action, optionally guarded by a `when` condition.
/// Nesting (the old `conditional` block) is expressed as a guarded `scene`.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Light {
        // accepts a device alias (resolved at load time) or a raw address;
        // `ieeeAddr` kept as an alias for backwards compatibility with the HTTP execute route
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
        state: LightState,
        #[serde(default)]
        when: Option<Condition>,
    },
    Switch {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[allow(unused)]
        state: SwitchState,
        #[serde(default)]
        when: Option<Condition>,
    },
    Scene {
        run: Vec<Step>,
        #[serde(default)]
        when: Option<Condition>,
    },
    Notify {
        notify: NotifySource,
        message: String,
        #[serde(default)]
        when: Option<Condition>,
    },
    Delay {
        seconds: u64,
        #[serde(default)]
        when: Option<Condition>,
    },
    RunWorkflow {
        workflow: String,
        #[serde(default)]
        when: Option<Condition>,
    },
}

impl Step {
    /// Static step kind, used as a label in logs, spans, and metrics.
    pub fn kind(&self) -> &'static str {
        match self {
            Step::Light { .. } => "light",
            Step::Switch { .. } => "switch",
            Step::Scene { .. } => "scene",
            Step::Notify { .. } => "notify",
            Step::Delay { .. } => "delay",
            Step::RunWorkflow { .. } => "run_workflow",
        }
    }

    /// The optional guard condition shared across every step variant.
    pub fn guard(&self) -> Option<&Condition> {
        match self {
            Step::Light { when, .. }
            | Step::Switch { when, .. }
            | Step::Scene { when, .. }
            | Step::Notify { when, .. }
            | Step::Delay { when, .. }
            | Step::RunWorkflow { when, .. } => when.as_ref(),
        }
    }

    pub fn describe_action(&self) -> Option<String> {
        match self {
            Step::Light {
                ieee_addr, state, ..
            } => Some(format!("light({ieee_addr}) -> {state:?}")),
            Step::Switch {
                ieee_addr, state, ..
            } => Some(format!("switch({ieee_addr}) -> {state:?}")),
            Step::Notify {
                notify, message, ..
            } => Some(format!("notify({notify:?}): {message}")),
            Step::Delay { seconds, .. } => Some(format!("delay {seconds}s")),
            Step::Scene { .. } | Step::RunWorkflow { .. } => None,
        }
    }

    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Step::Light {
                ieee_addr, when, ..
            }
            | Step::Switch {
                ieee_addr, when, ..
            } => {
                validate_device(ieee_addr, devices)?;
                resolve_opt(when, devices)?;
            }
            Step::Scene { run, when } => {
                for step in run {
                    step.resolve_devices(devices)?;
                }
                resolve_opt(when, devices)?;
            }
            Step::Notify { when, .. }
            | Step::Delay { when, .. }
            | Step::RunWorkflow { when, .. } => resolve_opt(when, devices)?,
        }
        Ok(())
    }
}

fn resolve_opt(when: &mut Option<Condition>, devices: &DeviceAliases) -> Result<(), String> {
    if let Some(when) = when {
        when.resolve_devices(devices)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum WorkflowTrigger {
    Triggered {
        on: TriggerMatcher,
        when: Option<Condition>,
        cooldown: Option<TimeDelta>,
        delay: Option<TimeDelta>,
    },
    Reusable,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawWorkflow")]
pub struct Workflow {
    pub name: String,
    pub enabled: bool,
    pub dry_run: bool,
    pub trigger: WorkflowTrigger,
    pub run: Vec<Step>,
}

#[derive(Debug, Deserialize, Clone)]
struct RawWorkflow {
    #[serde(default)]
    name: String,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    on: Option<TriggerMatcher>,
    #[serde(default)]
    when: Option<Condition>,
    #[serde(default, deserialize_with = "option_time_delta_from_str::deserialize")]
    cooldown: Option<TimeDelta>,
    #[serde(default, deserialize_with = "option_time_delta_from_str::deserialize")]
    delay: Option<TimeDelta>,
    run: Vec<Step>,
}

impl From<RawWorkflow> for Workflow {
    fn from(raw: RawWorkflow) -> Self {
        let trigger = match raw.on {
            Some(on) => WorkflowTrigger::Triggered {
                on,
                when: raw.when,
                cooldown: raw.cooldown,
                delay: raw.delay,
            },
            None => WorkflowTrigger::Reusable,
        };
        Workflow {
            name: raw.name,
            enabled: raw.enabled,
            dry_run: raw.dry_run,
            trigger,
            run: raw.run,
        }
    }
}

impl Workflow {
    pub fn on(&self) -> Option<&TriggerMatcher> {
        match &self.trigger {
            WorkflowTrigger::Triggered { on, .. } => Some(on),
            WorkflowTrigger::Reusable => None,
        }
    }

    pub fn when(&self) -> Option<&Condition> {
        match &self.trigger {
            WorkflowTrigger::Triggered { when, .. } => when.as_ref(),
            WorkflowTrigger::Reusable => None,
        }
    }

    pub fn cooldown(&self) -> Option<TimeDelta> {
        match &self.trigger {
            WorkflowTrigger::Triggered { cooldown, .. } => *cooldown,
            WorkflowTrigger::Reusable => None,
        }
    }

    pub fn delay(&self) -> Option<TimeDelta> {
        match &self.trigger {
            WorkflowTrigger::Triggered { delay, .. } => *delay,
            WorkflowTrigger::Reusable => None,
        }
    }

    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        if let WorkflowTrigger::Triggered { on, when, .. } = &mut self.trigger {
            on.resolve_devices(devices)?;
            if let Some(when) = when {
                when.resolve_devices(devices)?;
            }
        }
        for step in &mut self.run {
            step.resolve_devices(devices)?;
        }
        Ok(())
    }
}
