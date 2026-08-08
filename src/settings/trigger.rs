use schemars::JsonSchema;
use serde::Deserialize;

use super::workflow::Comparison;
use super::{DeviceAliases, IEEEAddress, validate_device};
use crate::actors::sun::calc::SunTransition;
use crate::actors::system::cron::schedule::CronSchedule;
use crate::event_bus::{PlaybackState, SensorMetric};
use crate::mode::Mode;

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
    /// Fires when a house mode is entered (`active: true`) or exited
    /// (`active: false`), driven by `set_mode`.
    Mode {
        mode: Mode,
        active: bool,
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

    /// Template variable names this trigger's event can supply to a `notify`
    /// message, mirroring [`crate::event_bus::EventBusMessage::vars`]. Used by the
    /// config loader to warn about `${unknown}` placeholders.
    pub fn available_vars(&self) -> Vec<String> {
        let strs = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect();
        match self {
            TriggerMatcher::Presence { .. } => strs(&["sensor", "present"]),
            TriggerMatcher::Door { .. } => strs(&["device", "open"]),
            TriggerMatcher::Switch { .. } => strs(&["device", "action"]),
            TriggerMatcher::Environment { metric, .. } => {
                vec![
                    "sensor".to_owned(),
                    crate::event_bus::metric_var_name(metric),
                ]
            }
            TriggerMatcher::Cron { .. } => strs(&["name"]),
            TriggerMatcher::Sun { .. } => strs(&["transition"]),
            TriggerMatcher::Mode { .. } => strs(&["mode", "active"]),
            TriggerMatcher::HomeAssistant { .. } => strs(&["entity_id", "state"]),
            TriggerMatcher::Woolworths { .. } => {
                strs(&["product_id", "name", "old_price", "new_price", "drop"])
            }
            TriggerMatcher::DeviceBattery { .. } => strs(&[
                "device_id",
                "kind",
                "name",
                "battery_voltage",
                "battery_percent",
            ]),
            TriggerMatcher::Jellyfin { .. } => strs(&[
                "state",
                "session_id",
                "user",
                "device",
                "client",
                "item",
                "item_type",
                "series",
                "season",
                "episode",
                "position",
                "runtime",
                "play_method",
            ]),
            TriggerMatcher::MediaPlayer { .. } => strs(&[
                "device",
                "name",
                "room",
                "state",
                "entity_state",
                "app",
                "source",
                "item",
                "series",
                "item_type",
                "season",
                "episode",
                "position",
                "duration",
                "volume",
                "muted",
            ]),
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
            | TriggerMatcher::HomeAssistant { .. }
            | TriggerMatcher::Woolworths { .. }
            | TriggerMatcher::DeviceBattery { .. }
            | TriggerMatcher::Jellyfin { .. }
            | TriggerMatcher::MediaPlayer { .. } => {}
        }
        Ok(())
    }
}
