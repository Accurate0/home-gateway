//! In-memory event bus.
//!
//! Producers anywhere in the app publish an [`EventBusMessage`] without knowing
//! which (if any) workflows it triggers. The [`crate::actors::workflows::dispatcher`]
//! actor subscribes, matches the message against the configured `triggers:`, and
//! forwards work to the parallel workflow factory. The bus itself does no
//! matching or execution — it is a thin fan-out wrapper around a tokio broadcast
//! channel, cloned onto [`crate::types::SharedActorState`].

use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::actors::sun::calc::SunTransition;
use crate::mode::Mode;
use crate::settings::IEEEAddress;

/// A named scalar sensor reading. Known metrics are typed; anything else (an
/// esphome object_id we don't model) falls back to [`SensorMetric::Other`] so
/// producers and config stay honest without an exhaustive list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(from = "String")]
#[schemars(with = "String")]
pub enum SensorMetric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
    SoilMoisture,
    Other(String),
}

impl From<String> for SensorMetric {
    fn from(s: String) -> Self {
        match s.as_str() {
            "temperature" => SensorMetric::Temperature,
            "humidity" => SensorMetric::Humidity,
            "pressure" => SensorMetric::Pressure,
            "lux" => SensorMetric::Lux,
            "uv_index" => SensorMetric::UvIndex,
            "soil_moisture" => SensorMetric::SoilMoisture,
            _ => SensorMetric::Other(s),
        }
    }
}

/// A sensor reading: which metric, and its scalar value. The runtime companion
/// to [`SensorMetric`] (which is the value-less discriminant used in config).
#[derive(Debug, Clone, PartialEq)]
pub enum SensorReading {
    Temperature { value: f64 },
    Humidity { value: f64 },
    Pressure { value: f64 },
    Lux { value: f64 },
    UvIndex { value: f64 },
    SoilMoisture { value: f64 },
    Other { name: String, value: f64 },
}

/// Template variable name for a metric, matching the config metric strings
/// (`temperature`, `soil_moisture`, …) so `${temperature}` works in a notify.
pub fn metric_var_name(metric: &SensorMetric) -> String {
    match metric {
        SensorMetric::Temperature => "temperature".to_owned(),
        SensorMetric::Humidity => "humidity".to_owned(),
        SensorMetric::Pressure => "pressure".to_owned(),
        SensorMetric::Lux => "lux".to_owned(),
        SensorMetric::UvIndex => "uv_index".to_owned(),
        SensorMetric::SoilMoisture => "soil_moisture".to_owned(),
        SensorMetric::Other(name) => name.clone(),
    }
}

impl SensorReading {
    /// Build a reading from a metric discriminant and value (e.g. mapping a raw
    /// esphome object_id through [`SensorMetric::from`]).
    pub fn new(metric: SensorMetric, value: f64) -> Self {
        match metric {
            SensorMetric::Temperature => SensorReading::Temperature { value },
            SensorMetric::Humidity => SensorReading::Humidity { value },
            SensorMetric::Pressure => SensorReading::Pressure { value },
            SensorMetric::Lux => SensorReading::Lux { value },
            SensorMetric::UvIndex => SensorReading::UvIndex { value },
            SensorMetric::SoilMoisture => SensorReading::SoilMoisture { value },
            SensorMetric::Other(name) => SensorReading::Other { name, value },
        }
    }

    pub fn metric(&self) -> SensorMetric {
        match self {
            SensorReading::Temperature { .. } => SensorMetric::Temperature,
            SensorReading::Humidity { .. } => SensorMetric::Humidity,
            SensorReading::Pressure { .. } => SensorMetric::Pressure,
            SensorReading::Lux { .. } => SensorMetric::Lux,
            SensorReading::UvIndex { .. } => SensorMetric::UvIndex,
            SensorReading::SoilMoisture { .. } => SensorMetric::SoilMoisture,
            SensorReading::Other { name, .. } => SensorMetric::Other(name.clone()),
        }
    }

    pub fn value(&self) -> f64 {
        match self {
            SensorReading::Temperature { value }
            | SensorReading::Humidity { value }
            | SensorReading::Pressure { value }
            | SensorReading::Lux { value }
            | SensorReading::UvIndex { value }
            | SensorReading::SoilMoisture { value }
            | SensorReading::Other { value, .. } => *value,
        }
    }
}

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
    /// and is owned by the [`crate::actors::cron::CronActor`] producer.
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
    /// `state_changed` stream by the [`crate::actors::home_assistant`] producer.
    HomeAssistant {
        event_id: Uuid,
        entity_id: String,
        state: String,
    },
    /// A tracked Woolworths product dropped in price, published by the
    /// [`crate::actors::woolworths`] producer so workflows can trigger on it.
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
        battery_voltage: f64,
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
                ..
            } => HashMap::from([
                ("device_id".to_owned(), device_id.clone()),
                ("kind".to_owned(), kind.clone()),
                ("name".to_owned(), name.clone()),
                (
                    "battery_voltage".to_owned(),
                    format!("{battery_voltage:.3}"),
                ),
            ]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventFilter {
    All,
    Parts {
        domain: FilterSegment,
        entity: FilterSegment,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterSegment {
    Any,
    Exact(String),
}

impl FilterSegment {
    fn parse(s: &str) -> Self {
        if s == "*" {
            FilterSegment::Any
        } else {
            FilterSegment::Exact(s.to_owned())
        }
    }

    fn matches(&self, other: &str) -> bool {
        match self {
            FilterSegment::Any => true,
            FilterSegment::Exact(value) => value == other,
        }
    }
}

impl EventFilter {
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw == "*" {
            return Some(EventFilter::All);
        }

        let mut segments = raw.split(':');
        let domain = segments.next()?;
        let entity = segments.next()?;
        if segments.next().is_some() {
            return None;
        }

        let domain = FilterSegment::parse(domain);
        if let FilterSegment::Exact(domain) = &domain
            && !EventBusMessage::KINDS.contains(&domain.as_str())
        {
            return None;
        }

        Some(EventFilter::Parts {
            domain,
            entity: FilterSegment::parse(entity),
        })
    }

    pub fn matches(&self, msg: &EventBusMessage) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Parts { domain, entity } => {
                domain.matches(msg.kind()) && entity.matches(&msg.entity())
            }
        }
    }

    pub fn domains(&self) -> Vec<&str> {
        match self {
            EventFilter::All
            | EventFilter::Parts {
                domain: FilterSegment::Any,
                ..
            } => EventBusMessage::KINDS.to_vec(),
            EventFilter::Parts {
                domain: FilterSegment::Exact(domain),
                ..
            } => vec![domain.as_str()],
        }
    }
}

/// Clonable handle to the in-memory event bus. Cheap to clone (shares one
/// broadcast sender). Stored on `SharedActorState` so any actor can publish.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<EventBusMessage>,
}

impl EventBus {
    /// `capacity` bounds the per-subscriber backlog; slow subscribers that fall
    /// behind observe a `Lagged` error rather than blocking producers.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event. Failure means there are currently no subscribers, which
    /// is not an error for a fire-and-forget bus.
    pub fn publish(&self, msg: EventBusMessage) {
        let kind = msg.kind();
        let event_id = msg.event_id();
        if self.tx.send(msg).is_err() {
            tracing::debug!("[{event_id}] no subscribers for {kind} event");
        }
    }

    /// Subscribe a new receiver. Each subscriber sees every event published
    /// after it subscribed.
    pub fn subscribe(&self) -> broadcast::Receiver<EventBusMessage> {
        self.tx.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        // 1024 is generous for a home-automation event rate; lagging here would
        // mean ~1000 unhandled events backed up, which warrants the warning.
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presence(sensor: &str) -> EventBusMessage {
        EventBusMessage::Presence {
            event_id: Uuid::nil(),
            sensor: sensor.to_owned(),
            present: true,
        }
    }

    fn cron(name: &str) -> EventBusMessage {
        EventBusMessage::Cron {
            event_id: Uuid::nil(),
            name: name.to_owned(),
        }
    }

    fn matches(pattern: &str, msg: &EventBusMessage) -> bool {
        EventFilter::parse(pattern).unwrap().matches(msg)
    }

    #[test]
    fn exact_match() {
        assert!(matches("presence:livingroom", &presence("livingroom")));
        assert!(!matches("presence:livingroom", &presence("kitchen")));
        assert!(!matches("presence:livingroom", &cron("nightly")));
    }

    #[test]
    fn entity_wildcard() {
        assert!(matches("presence:*", &presence("livingroom")));
        assert!(matches("presence:*", &presence("kitchen")));
        assert!(!matches("presence:*", &cron("nightly")));
    }

    #[test]
    fn domain_wildcard() {
        assert!(matches("*:livingroom", &presence("livingroom")));
        assert!(!matches("*:livingroom", &presence("kitchen")));
    }

    #[test]
    fn global_wildcard() {
        assert!(matches("*", &presence("livingroom")));
        assert!(matches("*", &cron("nightly")));
    }

    #[test]
    fn invalid_filters_do_not_parse() {
        assert!(EventFilter::parse("presence").is_none());
        assert!(EventFilter::parse("presence:living:extra").is_none());
        assert!(EventFilter::parse("bogus:living").is_none());
    }
}
