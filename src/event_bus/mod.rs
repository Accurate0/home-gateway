//! In-memory event bus.
//!
//! Producers anywhere in the app publish an [`EventBusMessage`] without knowing
//! which (if any) workflows it triggers. The [`crate::actors::workflows::dispatcher`]
//! actor subscribes, matches the message against the configured `triggers:`, and
//! forwards work to the parallel workflow factory. The bus itself does no
//! matching or execution — it is a thin fan-out wrapper around a tokio broadcast
//! channel, cloned onto [`crate::state::AppState`].

pub mod bus;
pub mod filter;
pub mod message;
pub mod playback;
pub mod reading;
pub mod solar_metric;

pub use bus::EventBus;
pub use filter::{EventFilter, FilterSegment};
pub use message::EventBusMessage;
pub use playback::PlaybackState;
pub use reading::{SensorMetric, SensorReading, metric_var_name};
pub use solar_metric::SolarMetric;
