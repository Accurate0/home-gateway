use chrono::{NaiveTime, TimeDelta};
use schemars::JsonSchema;
use serde::Deserialize;

use super::{CompareOp, Comparison, EnvMetric, SwitchMetric};
use crate::actors::sun::calc::SunPeriod;
use crate::event_bus::{ForecastDay, SolarMetric, WeatherMetric, WeatherSource};
use crate::mode::Mode;
use crate::settings::{DeviceAliases, IEEEAddress, validate_device};
use crate::templating::{Expr, Literal};

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LeafCondition {
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
        #[schemars(with = "String")]
        offset: TimeDelta,
    },
    Mode {
        is: Mode,
    },
    Solar {
        metric: SolarMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    HomeAssistant {
        entity_id: String,
        state: String,
    },
    SmartSwitch {
        #[serde(rename = "device")]
        ieee_addr: IEEEAddress,
        metric: SwitchMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    Weather {
        source: WeatherSource,
        metric: WeatherMetric,
        #[serde(default)]
        day: Option<ForecastDay>,
        #[serde(flatten)]
        cmp: Comparison,
    },
    Var {
        var: Expr,
        op: CompareOp,
        value: Literal,
    },
}

impl LeafCondition {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            LeafCondition::Light { ieee_addr, .. }
            | LeafCondition::Door { ieee_addr, .. }
            | LeafCondition::SmartSwitch { ieee_addr, .. } => {
                validate_device(ieee_addr, devices)?;
            }
            LeafCondition::Environment { .. }
            | LeafCondition::Presence { .. }
            | LeafCondition::TimeOfDay { .. }
            | LeafCondition::Mode { .. }
            | LeafCondition::Sun { .. }
            | LeafCondition::Solar { .. }
            | LeafCondition::HomeAssistant { .. }
            | LeafCondition::Var { .. } => {}
            LeafCondition::Weather {
                source,
                metric,
                day,
                ..
            } => metric.validate(*source, *day)?,
        }
        Ok(())
    }

    pub(super) fn describe(&self) -> String {
        match self {
            LeafCondition::Light { ieee_addr, on } => {
                format!("light({ieee_addr}) is {}", if *on { "on" } else { "off" })
            }
            LeafCondition::Environment {
                sensor,
                metric,
                cmp,
            } => format!("env({sensor}).{metric:?} {:?} {}", cmp.op, cmp.value),
            LeafCondition::Presence { sensor, present } => {
                format!("presence({sensor}) is {present}")
            }
            LeafCondition::Door { ieee_addr, open } => {
                format!(
                    "door({ieee_addr}) is {}",
                    if *open { "open" } else { "closed" }
                )
            }
            LeafCondition::TimeOfDay { after, before } => match (after, before) {
                (Some(a), Some(b)) => format!("time in [{a}, {b})"),
                (Some(a), None) => format!("time after {a}"),
                (None, Some(b)) => format!("time before {b}"),
                (None, None) => "time always".to_string(),
            },
            LeafCondition::Sun { is, offset } => {
                if offset.is_zero() {
                    format!("sun is {is:?}")
                } else {
                    format!(
                        "sun is {is:?} (offset {})",
                        crate::timedelta_format::humanize(*offset)
                    )
                }
            }
            LeafCondition::Mode { is } => format!("mode is {}", is.as_str()),
            LeafCondition::Solar { metric, cmp } => {
                format!("solar.{metric} {:?} {}", cmp.op, cmp.value)
            }
            LeafCondition::HomeAssistant { entity_id, state } => {
                format!("ha({entity_id}) is {state}")
            }
            LeafCondition::SmartSwitch {
                ieee_addr,
                metric,
                cmp,
            } => format!(
                "switch({ieee_addr}).{} {:?} {}",
                metric.as_str(),
                cmp.op,
                cmp.value
            ),
            LeafCondition::Weather {
                source,
                metric,
                day,
                cmp,
            } => format!(
                "weather({}).{} {:?} {}",
                source.as_str(),
                metric.label(*day),
                cmp.op,
                cmp.value
            ),
            LeafCondition::Var { var, op, value } => format!("{var} {op:?} {value}"),
        }
    }
}
