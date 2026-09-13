use schemars::JsonSchema;
use serde::Deserialize;

use super::{Comparison, EnvMetric, LeafCondition};
use crate::actors::sun::calc::SunTransition;
use crate::actors::system::cron::schedule::CronSchedule;
use crate::event_bus::{
    ForecastDay, FuelChange, PlaybackState, SensorMetric, SolarMetric, WeatherMetric, WeatherSource,
};
use crate::mode::Mode;
use crate::settings::{DeviceAliases, IEEEAddress, validate_device};

/// Which event a trigger fires on. Mirrors the [`crate::event_bus::EventBusMessage`]
/// variants; the dispatcher matches messages against these.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
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
        #[serde(default)]
        action: Option<String>,
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
    /// [`crate::actors::system::cron::CronActor`] producer, which matches by trigger name.
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
        #[schemars(with = "String")]
        offset: chrono::TimeDelta,
    },
    Mode {
        #[serde(default)]
        to: Option<Mode>,
        #[serde(default)]
        from: Option<Mode>,
    },
    /// Fires when a Home Assistant entity changes state, driven by the
    /// [`crate::actors::integrations::home_assistant`] producer. Optionally gate on the entity
    /// reaching a specific `state`.
    HomeAssistant {
        entity_id: String,
        #[serde(default)]
        state: Option<String>,
    },
    /// Fires when a tracked Woolworths product drops in price, driven by the
    /// [`crate::actors::integrations::woolworths`] producer. Optionally gate on a specific
    /// `product_id` and/or a minimum drop amount (in dollars).
    Woolworths {
        #[serde(default)]
        product_id: Option<i64>,
        #[serde(default)]
        min_drop: Option<f64>,
    },
    #[serde(rename = "fuelwatch")]
    FuelWatch {
        change: FuelChange,
        #[serde(default)]
        site_id: Option<i32>,
        #[serde(default)]
        below: Option<f64>,
        #[serde(default)]
        min_drop: Option<f64>,
    },
    /// Fires when a poll-transport device reports its battery voltage on
    /// check-in. Optionally gate on a specific `device_id`, device `kind`,
    /// and/or a `below` voltage threshold for low-battery alerts.
    DeviceBattery {
        #[serde(default)]
        device_id: Option<String>,
        #[serde(default)]
        kind: Option<String>,
        #[serde(default)]
        below: Option<f64>,
    },
    /// Fires on a Jellyfin playback edge, driven by the
    /// [`crate::actors::integrations::jellyfin`] producer. Every field is an
    /// optional gate: `state` (`started`/`stopped`/`paused`/`resumed`), the
    /// Jellyfin `user`, the playing `device`, and the Jellyfin item `item_type`
    /// (e.g. `Movie`, `Episode`, `Audio`).
    Jellyfin {
        #[serde(default)]
        state: Option<PlaybackState>,
        #[serde(default)]
        user: Option<String>,
        #[serde(default)]
        device: Option<String>,
        #[serde(default)]
        item_type: Option<String>,
    },
    /// Fires on a `media_player` playback edge, driven by the
    /// [`crate::actors::devices::media_player`] handler. Every field is an optional
    /// gate: the configured `device` id, `state`
    /// (`started`/`stopped`/`paused`/`resumed`), and the casting `app` (e.g.
    /// `Jellyfin`, `YouTube`).
    MediaPlayer {
        #[serde(default)]
        device: Option<String>,
        #[serde(default)]
        state: Option<PlaybackState>,
        #[serde(default)]
        app: Option<String>,
    },
    /// Fires on a solar generation reading, driven by the
    /// [`crate::actors::integrations::solar`] producer's poll. `metric` picks the
    /// live reading (`current`) or a rolling average (`avg_15m`, `avg_1h`,
    /// `avg_3h`); the flattened comparison is the threshold, in watts. As with
    /// [`TriggerMatcher::Environment`], the dispatcher fires on the rising edge.
    Solar {
        metric: SolarMetric,
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
}

impl TriggerMatcher {
    pub fn event_kind(&self) -> &'static str {
        match self {
            TriggerMatcher::Presence { .. } => "presence",
            TriggerMatcher::Door { .. } => "door",
            TriggerMatcher::Switch { .. } => "switch",
            TriggerMatcher::Environment { .. } => "environment",
            TriggerMatcher::Cron { .. } => "cron",
            TriggerMatcher::Sun { .. } => "sun",
            TriggerMatcher::Mode { .. } => "mode",
            TriggerMatcher::HomeAssistant { .. } => "home_assistant",
            TriggerMatcher::Woolworths { .. } => "woolworths",
            TriggerMatcher::FuelWatch { .. } => "fuelwatch",
            TriggerMatcher::DeviceBattery { .. } => "device_battery",
            TriggerMatcher::Jellyfin { .. } => "jellyfin",
            TriggerMatcher::MediaPlayer { .. } => "media_player",
            TriggerMatcher::Solar { .. } => "solar",
            TriggerMatcher::Weather { .. } => "weather",
        }
    }

    pub fn supports_hold(&self) -> bool {
        self.as_condition().is_some()
    }

    pub fn as_condition(&self) -> Option<LeafCondition> {
        match self {
            TriggerMatcher::Presence { sensor, present } => Some(LeafCondition::Presence {
                sensor: sensor.clone(),
                present: *present,
            }),
            TriggerMatcher::Door { ieee_addr, open } => Some(LeafCondition::Door {
                ieee_addr: ieee_addr.clone(),
                open: *open,
            }),
            TriggerMatcher::Environment {
                sensor,
                metric,
                cmp,
            } => {
                let metric = match metric {
                    SensorMetric::Temperature => EnvMetric::Temperature,
                    SensorMetric::Humidity => EnvMetric::Humidity,
                    SensorMetric::Pressure => EnvMetric::Pressure,
                    SensorMetric::Lux => EnvMetric::Lux,
                    SensorMetric::UvIndex => EnvMetric::UvIndex,
                    SensorMetric::Pm25
                    | SensorMetric::VocIndex
                    | SensorMetric::SoilMoisture
                    | SensorMetric::Other(_) => return None,
                };

                Some(LeafCondition::Environment {
                    sensor: sensor.clone(),
                    metric,
                    cmp: *cmp,
                })
            }
            TriggerMatcher::Solar { metric, cmp } => Some(LeafCondition::Solar {
                metric: *metric,
                cmp: *cmp,
            }),
            TriggerMatcher::HomeAssistant {
                entity_id,
                state: Some(state),
            } => Some(LeafCondition::HomeAssistant {
                entity_id: entity_id.clone(),
                state: state.clone(),
            }),
            TriggerMatcher::Weather {
                source,
                metric,
                day,
                cmp,
            } => Some(LeafCondition::Weather {
                source: *source,
                metric: *metric,
                day: *day,
                cmp: *cmp,
            }),
            TriggerMatcher::HomeAssistant { state: None, .. }
            | TriggerMatcher::Switch { .. }
            | TriggerMatcher::Cron { .. }
            | TriggerMatcher::Sun { .. }
            | TriggerMatcher::Mode { .. }
            | TriggerMatcher::Woolworths { .. }
            | TriggerMatcher::FuelWatch { .. }
            | TriggerMatcher::DeviceBattery { .. }
            | TriggerMatcher::Jellyfin { .. }
            | TriggerMatcher::MediaPlayer { .. } => None,
        }
    }

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
            TriggerMatcher::Switch {
                ieee_addr,
                action: Some(action),
            } => format!("switch({ieee_addr}) action={action}"),
            TriggerMatcher::Switch {
                ieee_addr,
                action: None,
            } => format!("switch({ieee_addr}) any action"),
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
            TriggerMatcher::Mode { to, from } => {
                let side = |mode: &Option<Mode>| mode.map_or("*", |mode| mode.as_str());

                format!("mode({} -> {})", side(from), side(to))
            }
            TriggerMatcher::HomeAssistant { entity_id, state } => match state {
                Some(state) => format!("home_assistant({entity_id}) -> {state}"),
                None => format!("home_assistant({entity_id})"),
            },
            TriggerMatcher::Woolworths {
                product_id,
                min_drop,
            } => {
                let product = product_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "*".to_owned());
                match min_drop {
                    Some(min) => format!("woolworths({product}) drop >= {min}"),
                    None => format!("woolworths({product}) price drop"),
                }
            }
            TriggerMatcher::FuelWatch {
                change,
                site_id,
                below,
                min_drop,
            } => {
                let site = site_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "*".to_owned());
                let mut out = format!("fuelwatch({site}) {}", change.as_str());

                if let Some(below) = below {
                    out.push_str(&format!(" below {below}"));
                }

                if let Some(min) = min_drop {
                    out.push_str(&format!(" drop >= {min}"));
                }

                out
            }
            TriggerMatcher::DeviceBattery {
                device_id,
                kind,
                below,
            } => {
                let device = device_id
                    .clone()
                    .or_else(|| kind.clone())
                    .unwrap_or_else(|| "*".to_owned());
                match below {
                    Some(v) => format!("device_battery({device}) < {v}"),
                    None => format!("device_battery({device})"),
                }
            }
            TriggerMatcher::Jellyfin {
                state,
                user,
                device,
                item_type,
            } => {
                let subject = user
                    .clone()
                    .or_else(|| device.clone())
                    .or_else(|| item_type.clone())
                    .unwrap_or_else(|| "*".to_owned());
                match state {
                    Some(state) => format!("jellyfin({subject}) -> {}", state.as_str()),
                    None => format!("jellyfin({subject})"),
                }
            }
            TriggerMatcher::MediaPlayer { device, state, app } => {
                let subject = device
                    .clone()
                    .or_else(|| app.clone())
                    .unwrap_or_else(|| "*".to_owned());
                match state {
                    Some(state) => format!("media_player({subject}) -> {}", state.as_str()),
                    None => format!("media_player({subject})"),
                }
            }
            TriggerMatcher::Solar { metric, cmp } => {
                format!("solar.{metric} {:?} {}", cmp.op, cmp.value)
            }
            TriggerMatcher::Weather {
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

    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            TriggerMatcher::Door { ieee_addr, .. } | TriggerMatcher::Switch { ieee_addr, .. } => {
                validate_device(ieee_addr, devices)?;
            }
            TriggerMatcher::Presence { sensor, .. }
            | TriggerMatcher::Environment { sensor, .. } => {
                validate_device(sensor, devices)?;
            }
            TriggerMatcher::Mode {
                to: None,
                from: None,
            } => {
                return Err("mode trigger needs `to` or `from`".to_owned());
            }
            TriggerMatcher::Cron { .. }
            | TriggerMatcher::Sun { .. }
            | TriggerMatcher::Mode { .. }
            | TriggerMatcher::HomeAssistant { .. }
            | TriggerMatcher::Woolworths { .. }
            | TriggerMatcher::FuelWatch { .. }
            | TriggerMatcher::DeviceBattery { .. }
            | TriggerMatcher::Jellyfin { .. }
            | TriggerMatcher::MediaPlayer { .. }
            | TriggerMatcher::Solar { .. } => {}
            TriggerMatcher::Weather {
                source,
                metric,
                day,
                ..
            } => metric.validate(*source, *day)?,
        }
        Ok(())
    }
}
