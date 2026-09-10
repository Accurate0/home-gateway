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
pub mod forecast_day;
pub mod fuel_change;
pub mod message;
pub mod playback;
pub mod reading;
pub mod solar_metric;
pub mod subscriber;
pub mod weather_metric;
pub mod weather_reading;
pub mod weather_source;

pub use bus::EventBus;
pub use filter::{EventFilter, FilterSegment};
pub use forecast_day::ForecastDay;
pub use fuel_change::FuelChange;
pub use message::{BusEvent, EventBusMessage, FeatureFlagState};
pub use playback::PlaybackState;
pub use reading::{SensorMetric, SensorReading, metric_var_name};
pub use solar_metric::SolarMetric;
pub use subscriber::{EventSubscriber, Recipient, Subscription};
pub use weather_metric::WeatherMetric;
pub use weather_reading::WeatherReading;
pub use weather_source::WeatherSource;
