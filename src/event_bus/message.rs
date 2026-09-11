use uuid::Uuid;

use super::fuel_change::FuelChange;
use super::playback::PlaybackState;
use super::reading::SensorReading;
use super::variables::{
    CronVariables, DeviceBatteryVariables, DoorVariables, EnvironmentVariables, FuelWatchVariables,
    HomeAssistantVariables, JellyfinVariables, MediaPlayerVariables, ModeVariables,
    PresenceVariables, SolarVariables, SunVariables, SwitchVariables, WeatherVariables,
    WoolworthsVariables,
};
use super::weather_reading::WeatherReading;
use super::weather_source::WeatherSource;
use crate::actors::sun::calc::SunTransition;
use crate::mode::Mode;
use crate::repo::intent::{DeviceKind, IntentAttributes};
use crate::settings::IEEEAddress;
use crate::variables::{Node, WorkflowContextVariables};

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
        brightness: Option<i32>,
        colour_temp: Option<i32>,
        colour: Option<String>,
    },
    /// A UniFi WiFi client connected or disconnected. `client` is the mapped
    /// friendly name (or `unknown`); published for subscribers, not triggered on.
    Unifi {
        event_id: Uuid,
        mac_address: String,
        client: String,
        connected: bool,
    },
    Mode {
        event_id: Uuid,
        mode: Mode,
        previous: Mode,
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
    /// A Jellyfin playback session started, stopped, paused or resumed, derived by
    /// the [`crate::actors::integrations::jellyfin`] producer from the session
    /// snapshots it receives over the WebSocket and the `/Sessions` poll. Progress
    /// within an item is deliberately not published — only the edges are.
    Jellyfin {
        event_id: Uuid,
        state: PlaybackState,
        session_id: String,
        user: String,
        device: String,
        client: String,
        item_id: String,
        item_name: String,
        item_type: String,
        series_name: Option<String>,
        season: Option<i32>,
        episode: Option<i32>,
        position_seconds: Option<f64>,
        runtime_seconds: Option<f64>,
        play_method: Option<String>,
    },
    /// A Home Assistant `media_player` entity reached a playback edge, derived by
    /// the [`crate::actors::devices::media_player`] handler from HA `state_changed`
    /// events. As with [`EventBusMessage::Jellyfin`], progress within an item is
    /// deliberately not published — only the edges are.
    MediaPlayer {
        event_id: Uuid,
        device_id: String,
        name: String,
        room: Option<String>,
        state: PlaybackState,
        entity_state: String,
        app_name: Option<String>,
        source: Option<String>,
        media_title: Option<String>,
        media_series_title: Option<String>,
        media_content_type: Option<String>,
        season: Option<i32>,
        episode: Option<i32>,
        position_seconds: Option<f64>,
        duration_seconds: Option<f64>,
        volume_level: Option<f64>,
        muted: Option<bool>,
        artwork_url: Option<String>,
    },
    /// A solar generation reading, published by the
    /// [`crate::actors::integrations::solar`] producer after each poll. Only the
    /// live reading travels on the bus; the rolling averages are read from the DB
    /// on demand by the dispatcher, and only when a solar trigger asks for one.
    /// Threshold + rising-edge handling lives in the dispatcher, as it does for
    /// [`EventBusMessage::Environment`].
    Solar { event_id: Uuid, current_wh: f64 },
    Weather {
        event_id: Uuid,
        source: WeatherSource,
        readings: Vec<WeatherReading>,
    },
    FuelWatch {
        event_id: Uuid,
        change: FuelChange,
        site_id: i32,
        name: String,
        brand: String,
        suburb: String,
        address: String,
        old_price: f64,
        new_price: f64,
    },
    CommandFailed {
        event_id: Uuid,
        kind: DeviceKind,
        address: String,
        device_id: Option<String>,
        attributes: IntentAttributes,
        attempts: i32,
    },
    FeatureFlag {
        event_id: Uuid,
        state: FeatureFlagState,
        version: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureFlagState {
    Ready,
    Changed,
    Stale,
    Error,
}

impl FeatureFlagState {
    pub fn should_reevaluate(&self) -> bool {
        matches!(self, Self::Ready | Self::Changed)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Changed => "changed",
            Self::Stale => "stale",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BusEvent {
    pub traceparent: crate::tracing_context::TraceParent,
    pub message: EventBusMessage,
}

impl BusEvent {
    pub fn current(message: EventBusMessage) -> Self {
        Self {
            traceparent: crate::tracing_context::inject_current(),
            message,
        }
    }

    pub fn detached(message: EventBusMessage) -> Self {
        Self {
            traceparent: None,
            message,
        }
    }

    pub fn event_id(&self) -> Uuid {
        self.message.event_id()
    }

    pub fn kind(&self) -> &'static str {
        self.message.kind()
    }
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
            | EventBusMessage::DeviceBattery { event_id, .. }
            | EventBusMessage::Jellyfin { event_id, .. }
            | EventBusMessage::MediaPlayer { event_id, .. }
            | EventBusMessage::Solar { event_id, .. }
            | EventBusMessage::Weather { event_id, .. }
            | EventBusMessage::FuelWatch { event_id, .. }
            | EventBusMessage::CommandFailed { event_id, .. }
            | EventBusMessage::FeatureFlag { event_id, .. } => *event_id,
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
            EventBusMessage::Jellyfin { .. } => "jellyfin",
            EventBusMessage::MediaPlayer { .. } => "media_player",
            EventBusMessage::Solar { .. } => "solar",
            EventBusMessage::Weather { .. } => "weather",
            EventBusMessage::FuelWatch { .. } => "fuelwatch",
            EventBusMessage::CommandFailed { .. } => "command_failed",
            EventBusMessage::FeatureFlag { .. } => "feature_flag",
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
        "jellyfin",
        "media_player",
        "solar",
        "weather",
        "fuelwatch",
        "command_failed",
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
            EventBusMessage::Jellyfin { user, .. } => user.clone(),
            EventBusMessage::MediaPlayer { device_id, .. } => device_id.clone(),
            EventBusMessage::Solar { .. } => "solar".to_string(),
            EventBusMessage::Weather { source, .. } => source.as_str().to_owned(),
            EventBusMessage::FuelWatch { site_id, .. } => site_id.to_string(),
            EventBusMessage::CommandFailed { address, .. } => address.clone(),
            EventBusMessage::FeatureFlag { state, .. } => state.as_str().to_owned(),
        }
    }

    pub fn vars(&self) -> Node {
        match self {
            EventBusMessage::Presence {
                sensor, present, ..
            } => PresenceVariables {
                sensor: sensor.clone(),
                present: *present,
            }
            .to_node(),
            EventBusMessage::Door {
                ieee_addr, open, ..
            } => DoorVariables {
                device: ieee_addr.clone(),
                open: *open,
            }
            .to_node(),
            EventBusMessage::SwitchAction {
                ieee_addr, action, ..
            } => SwitchVariables {
                device: ieee_addr.clone(),
                action: action.clone(),
            }
            .to_node(),
            EventBusMessage::Environment {
                sensor, readings, ..
            } => EnvironmentVariables::node(sensor, readings),
            EventBusMessage::Cron { name, .. } => CronVariables { name: name.clone() }.to_node(),
            EventBusMessage::Sun { transition, .. } => SunVariables {
                transition: transition.as_str().to_owned(),
            }
            .to_node(),
            EventBusMessage::Mode { mode, previous, .. } => ModeVariables {
                mode: mode.as_str().to_owned(),
                previous: previous.as_str().to_owned(),
            }
            .to_node(),
            EventBusMessage::HomeAssistant {
                entity_id, state, ..
            } => HomeAssistantVariables {
                entity_id: entity_id.clone(),
                state: state.clone(),
            }
            .to_node(),
            EventBusMessage::Woolworths {
                product_id,
                name,
                old_price,
                new_price,
                ..
            } => WoolworthsVariables {
                product_id: *product_id,
                name: name.clone(),
                old_price: *old_price,
                new_price: *new_price,
                drop: old_price - new_price,
            }
            .to_node(),
            EventBusMessage::DeviceBattery {
                device_id,
                kind,
                name,
                battery_voltage,
                battery_percent,
                ..
            } => DeviceBatteryVariables {
                device_id: device_id.clone(),
                kind: kind.clone(),
                name: name.clone(),
                battery_voltage: *battery_voltage,
                battery_percent: *battery_percent,
            }
            .to_node(),
            EventBusMessage::Jellyfin {
                state,
                session_id,
                user,
                device,
                client,
                item_name,
                item_type,
                series_name,
                season,
                episode,
                position_seconds,
                runtime_seconds,
                play_method,
                ..
            } => JellyfinVariables {
                state: state.as_str().to_owned(),
                session_id: session_id.clone(),
                user: user.clone(),
                device: device.clone(),
                client: client.clone(),
                item: item_name.clone(),
                item_type: item_type.clone(),
                series: series_name.clone(),
                season: *season,
                episode: *episode,
                position: *position_seconds,
                runtime: *runtime_seconds,
                play_method: play_method.clone(),
            }
            .to_node(),
            EventBusMessage::MediaPlayer {
                device_id,
                name,
                room,
                state,
                entity_state,
                app_name,
                source,
                media_title,
                media_series_title,
                media_content_type,
                season,
                episode,
                position_seconds,
                duration_seconds,
                volume_level,
                muted,
                ..
            } => MediaPlayerVariables {
                device: device_id.clone(),
                name: name.clone(),
                room: room.clone(),
                state: state.as_str().to_owned(),
                entity_state: entity_state.clone(),
                app: app_name.clone(),
                source: source.clone(),
                item: media_title.clone(),
                series: media_series_title.clone(),
                item_type: media_content_type.clone(),
                season: *season,
                episode: *episode,
                position: *position_seconds,
                duration: *duration_seconds,
                volume: *volume_level,
                muted: *muted,
            }
            .to_node(),
            EventBusMessage::Solar { current_wh, .. } => {
                SolarVariables::new(*current_wh, None).to_node()
            }
            EventBusMessage::Weather {
                source, readings, ..
            } => WeatherVariables::node(*source, readings),
            EventBusMessage::FuelWatch {
                change,
                site_id,
                name,
                brand,
                suburb,
                address,
                old_price,
                new_price,
                ..
            } => FuelWatchVariables {
                change: change.as_str().to_owned(),
                site_id: *site_id,
                name: name.clone(),
                brand: brand.clone(),
                suburb: suburb.clone(),
                address: address.clone(),
                old_price: *old_price,
                new_price: *new_price,
                drop: old_price - new_price,
            }
            .to_node(),
            EventBusMessage::Light { .. }
            | EventBusMessage::Unifi { .. }
            | EventBusMessage::CommandFailed { .. }
            | EventBusMessage::FeatureFlag { .. } => Node::empty(),
        }
    }
}
