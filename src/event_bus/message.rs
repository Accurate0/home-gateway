use std::collections::HashMap;
use uuid::Uuid;

use super::reading::{SensorReading, metric_var_name};
use crate::actors::sun::calc::SunTransition;
use crate::mode::Mode;
use crate::settings::IEEEAddress;

/// Every event that can flow through the bus. New producers (webhooks,
/// schedules, manual triggers, …) add a variant here; matching lives in the
/// dispatcher and the `triggers:` config.
#[derive(Clone, Debug)]
pub enum EventBusMessage {
    /// A presence sensor transitioned (already edge-detected by the producer).
    Presence {
        event_id: Uuid,
        sensor: String,
        present: bool,
    },
    /// A door confirmed a state transition (debounced by `DerivedDoorEvents`).
    Door {
        event_id: Uuid,
        ieee_addr: IEEEAddress,
        open: bool,
    },
    /// A control switch / button reported an action (e.g. `single`, `on`).
    SwitchAction {
        event_id: Uuid,
        ieee_addr: IEEEAddress,
        action: String,
    },
    /// A bundle of scalar sensor readings captured together. An environment
    /// sensor emits all of its metrics (`temperature`, `humidity`, …) in a single
    /// event so every trigger and subscriber sees the full snapshot. Threshold +
    /// rising-edge handling lives in the dispatcher so a reading staying past a
    /// threshold only fires once.
    Environment {
        event_id: Uuid,
        sensor: String,
        readings: Vec<SensorReading>,
    },
    /// A scheduled `Cron` trigger came due. `name` identifies the trigger so the
    /// dispatcher can match it; the schedule itself lives in the trigger config
    /// and is owned by the [`crate::actors::system::cron::CronActor`] producer.
    Cron { event_id: Uuid, name: String },
    /// A sun transition (sunrise/sunset) came due, published by the
    /// [`crate::actors::sun::SunActor`] producer so workflows can trigger on dusk/dawn.
    Sun {
        event_id: Uuid,
        transition: SunTransition,
        offset: chrono::TimeDelta,
    },
    /// A light reported a power-state change (`on`/off), published for
    /// subscribers; the dispatcher does not currently trigger on it.
    Light {
        event_id: Uuid,
        ieee_addr: IEEEAddress,
        on: bool,
    },
    /// A UniFi WiFi client connected or disconnected. `client` is the mapped
    /// friendly name (or `unknown`); published for subscribers, not triggered on.
    Unifi {
        event_id: Uuid,
        mac_address: String,
        client: String,
        connected: bool,
    },
    /// A house mode was toggled (published per changed mode when `set_mode`
    /// runs), so transition workflows can trigger on enter/exit.
    Mode {
        event_id: Uuid,
        mode: Mode,
        active: bool,
    },
    /// A Home Assistant entity changed state, forwarded from HA's WebSocket
    /// `state_changed` stream by the [`crate::actors::integrations::home_assistant`] producer.
    HomeAssistant {
        event_id: Uuid,
        entity_id: String,
        state: String,
    },
    /// A tracked Woolworths product dropped in price, published by the
    /// [`crate::actors::integrations::woolworths`] producer so workflows can trigger on it.
    Woolworths {
        event_id: Uuid,
        product_id: i64,
        name: String,
        old_price: f64,
        new_price: f64,
    },
    /// A poll-transport device reported its battery voltage when it checked in
    /// (e.g. the eink display firmware hitting `/epd/config`), published so
    /// workflows can trigger a low-battery notification. `kind` is the device
    /// kind that reported it.
    DeviceBattery {
        event_id: Uuid,
        device_id: String,
        kind: String,
        name: String,
        battery_voltage: Option<f64>,
        battery_percent: Option<f64>,
    },
}

impl EventBusMessage {
    /// The correlation id carried by every event, for tracing through dispatch
    /// and workflow execution.
    pub fn event_id(&self) -> Uuid {
        match self {
            EventBusMessage::Presence { event_id, .. }
            | EventBusMessage::Door { event_id, .. }
            | EventBusMessage::SwitchAction { event_id, .. }
            | EventBusMessage::Environment { event_id, .. }
            | EventBusMessage::Cron { event_id, .. }
            | EventBusMessage::Sun { event_id, .. }
            | EventBusMessage::Light { event_id, .. }
            | EventBusMessage::Unifi { event_id, .. }
            | EventBusMessage::Mode { event_id, .. }
            | EventBusMessage::HomeAssistant { event_id, .. }
            | EventBusMessage::Woolworths { event_id, .. }
            | EventBusMessage::DeviceBattery { event_id, .. } => *event_id,
        }
    }

    /// A short static label for logs/metrics.
    pub fn kind(&self) -> &'static str {
        match self {
            EventBusMessage::Presence { .. } => "presence",
            EventBusMessage::Door { .. } => "door",
            EventBusMessage::SwitchAction { .. } => "switch",
            EventBusMessage::Environment { .. } => "environment",
            EventBusMessage::Cron { .. } => "cron",
            EventBusMessage::Sun { .. } => "sun",
            EventBusMessage::Light { .. } => "light",
            EventBusMessage::Unifi { .. } => "unifi",
            EventBusMessage::Mode { .. } => "mode",
            EventBusMessage::HomeAssistant { .. } => "home_assistant",
            EventBusMessage::Woolworths { .. } => "woolworths",
            EventBusMessage::DeviceBattery { .. } => "device_battery",
        }
    }

    pub const KINDS: &'static [&'static str] = &[
        "presence",
        "door",
        "switch",
        "environment",
        "cron",
        "sun",
        "light",
        "unifi",
        "mode",
        "home_assistant",
        "woolworths",
        "device_battery",
    ];

    pub fn entity(&self) -> String {
        match self {
            EventBusMessage::Presence { sensor, .. }
            | EventBusMessage::Environment { sensor, .. } => sensor.clone(),
            EventBusMessage::Door { ieee_addr, .. }
            | EventBusMessage::SwitchAction { ieee_addr, .. }
            | EventBusMessage::Light { ieee_addr, .. } => ieee_addr.to_string(),
            EventBusMessage::Cron { name, .. } => name.clone(),
            EventBusMessage::Sun { transition, .. } => match transition {
                SunTransition::Sunrise => "sunrise".to_string(),
                SunTransition::Sunset => "sunset".to_string(),
            },
            EventBusMessage::Unifi { mac_address, .. } => mac_address.clone(),
            EventBusMessage::Mode { mode, .. } => mode.as_str().to_string(),
            EventBusMessage::HomeAssistant { entity_id, .. } => entity_id.clone(),
            EventBusMessage::Woolworths { product_id, .. } => product_id.to_string(),
            EventBusMessage::DeviceBattery { device_id, .. } => device_id.clone(),
        }
    }

    /// Named variables carried by the event, substituted into templated workflow
    /// strings (e.g. a `notify` message). The keys each event kind can provide
    /// are declared in [`crate::settings::TriggerMatcher::available_vars`], which
    /// the config loader validates templates against.
    pub fn vars(&self) -> HashMap<String, String> {
        match self {
            EventBusMessage::Presence {
                sensor, present, ..
            } => HashMap::from([
                ("sensor".to_owned(), sensor.clone()),
                ("present".to_owned(), present.to_string()),
            ]),
            EventBusMessage::Door {
                ieee_addr, open, ..
            } => HashMap::from([
                ("device".to_owned(), ieee_addr.clone()),
                ("open".to_owned(), open.to_string()),
            ]),
            EventBusMessage::SwitchAction {
                ieee_addr, action, ..
            } => HashMap::from([
                ("device".to_owned(), ieee_addr.clone()),
                ("action".to_owned(), action.clone()),
            ]),
            EventBusMessage::Environment {
                sensor, readings, ..
            } => {
                let mut vars = HashMap::from([("sensor".to_owned(), sensor.clone())]);
                for reading in readings {
                    vars.insert(
                        metric_var_name(&reading.metric()),
                        reading.value().to_string(),
                    );
                }
                vars
            }
            EventBusMessage::Cron { name, .. } => {
                HashMap::from([("name".to_owned(), name.clone())])
            }
            EventBusMessage::Sun { transition, .. } => {
                HashMap::from([("transition".to_owned(), format!("{transition:?}"))])
            }
            EventBusMessage::Light { ieee_addr, on, .. } => HashMap::from([
                ("device".to_owned(), ieee_addr.clone()),
                ("on".to_owned(), on.to_string()),
            ]),
            EventBusMessage::Unifi {
                mac_address,
                client,
                connected,
                ..
            } => HashMap::from([
                ("mac_address".to_owned(), mac_address.clone()),
                ("client".to_owned(), client.clone()),
                ("connected".to_owned(), connected.to_string()),
            ]),
            EventBusMessage::Mode { mode, active, .. } => HashMap::from([
                ("mode".to_owned(), mode.as_str().to_owned()),
                ("active".to_owned(), active.to_string()),
            ]),
            EventBusMessage::HomeAssistant {
                entity_id, state, ..
            } => HashMap::from([
                ("entity_id".to_owned(), entity_id.clone()),
                ("state".to_owned(), state.clone()),
            ]),
            EventBusMessage::Woolworths {
                product_id,
                name,
                old_price,
                new_price,
                ..
            } => HashMap::from([
                ("product_id".to_owned(), product_id.to_string()),
                ("name".to_owned(), name.clone()),
                ("old_price".to_owned(), format!("{old_price:.2}")),
                ("new_price".to_owned(), format!("{new_price:.2}")),
                ("drop".to_owned(), format!("{:.2}", old_price - new_price)),
            ]),
            EventBusMessage::DeviceBattery {
                device_id,
                kind,
                name,
                battery_voltage,
                battery_percent,
                ..
            } => HashMap::from([
                ("device_id".to_owned(), device_id.clone()),
                ("kind".to_owned(), kind.clone()),
                ("name".to_owned(), name.clone()),
                (
                    "battery_voltage".to_owned(),
                    battery_voltage.map_or_else(String::new, |v| format!("{v:.3}")),
                ),
                (
                    "battery_percent".to_owned(),
                    battery_percent.map_or_else(String::new, |v| format!("{v:.0}")),
                ),
            ]),
        }
    }
}
