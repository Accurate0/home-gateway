//! In-memory event bus.
//!
//! Producers anywhere in the app publish an [`EventBusMessage`] without knowing
//! which (if any) workflows it triggers. The [`crate::actors::workflows::dispatcher`]
//! actor subscribes, matches the message against the configured `triggers:`, and
//! forwards work to the parallel workflow factory. The bus itself does no
//! matching or execution — it is a thin fan-out wrapper around a tokio broadcast
//! channel, cloned onto [`crate::types::SharedActorState`].

use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::actors::sun::calc::SunTransition;
use crate::settings::IEEEAddress;

/// A named scalar sensor reading. Known metrics are typed; anything else (an
/// esphome object_id we don't model) falls back to [`SensorMetric::Other`] so
/// producers and config stay honest without an exhaustive list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(from = "String")]
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
            | EventBusMessage::Unifi { event_id, .. } => *event_id,
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
