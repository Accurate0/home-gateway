//! In-memory event bus.
//!
//! Producers anywhere in the app publish an [`EventBusMessage`] without knowing
//! which (if any) workflows it triggers. The [`crate::actors::events::dispatcher`]
//! actor subscribes, matches the message against the configured `triggers:`, and
//! forwards work to the parallel workflow factory. The bus itself does no
//! matching or execution — it is a thin fan-out wrapper around a tokio broadcast
//! channel, cloned onto [`crate::types::SharedActorState`].

use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

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
    /// A scalar sensor reading. `metric` is the esphome object_id / reading name
    /// (`soil_moisture`, `temperature`, `humidity`, …). Threshold + rising-edge
    /// handling lives in the dispatcher so a reading staying past a threshold
    /// only fires once.
    Environment {
        event_id: Uuid,
        sensor: String,
        reading: SensorReading,
    },
    /// A scheduled `Cron` trigger came due. `name` identifies the trigger so the
    /// dispatcher can match it; the schedule itself lives in the trigger config
    /// and is owned by the [`crate::actors::cron::CronActor`] producer.
    Cron { event_id: Uuid, name: String },
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
            | EventBusMessage::Cron { event_id, .. } => *event_id,
        }
    }

    /// A short static label for logs/metrics.
    pub fn kind(&self) -> &'static str {
        match self {
            EventBusMessage::Presence { .. } => "presence",
            EventBusMessage::Door { .. } => "door",
            EventBusMessage::SwitchAction { .. } => "switch_action",
            EventBusMessage::Environment { .. } => "environment",
            EventBusMessage::Cron { .. } => "cron",
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
