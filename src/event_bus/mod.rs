//! In-memory event bus.
//!
//! Producers anywhere in the app publish an [`EventBusMessage`] without knowing
//! which (if any) workflows it triggers. The [`crate::actors::workflows::dispatcher`]
//! actor subscribes, matches the message against the configured `triggers:`, and
//! forwards work to the parallel workflow factory. The bus itself does no
//! matching or execution — it is a thin fan-out wrapper around a tokio broadcast
//! channel, cloned onto [`crate::state::AppState`].

pub mod bus;
pub mod custom_source;
pub mod feature_flag_state;
pub mod filter;
pub mod message;
pub mod reading;
pub mod subscriber;
pub mod variables;

pub use bus::EventBus;
pub use custom_source::CustomEventSource;
pub use feature_flag_state::FeatureFlagState;
pub use filter::{EventFilter, FilterSegment};
pub use message::{BusEvent, EventBusMessage};
pub use reading::{SensorMetric, SensorReading};
pub use subscriber::{EventSubscriber, Recipient, Subscription};
