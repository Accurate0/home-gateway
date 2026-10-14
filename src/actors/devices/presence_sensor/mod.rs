pub mod lua;
pub mod query;

use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

use crate::actors::devices::handler::DeviceHandler;
use crate::{event_bus::EventBusMessage, state::AppState};
use ractor::RpcReplyPort;
use uuid::Uuid;

pub struct Entity {
    pub address: String,
    pub sensor: Option<String>,
    pub presence: bool,
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
    pub traceparent: crate::telemetry::context::TraceParent,
}

pub enum Message {
    NewEvent(NewEvent),
    /// Last known presence for a sensor (keyed by ieee address or esphome node),
    /// or `None` if the sensor hasn't reported since startup.
    QueryLatest {
        sensor: String,
        traceparent: crate::telemetry::context::TraceParent,
        reply: RpcReplyPort<Option<bool>>,
    },
    QuerySeenWithin {
        sensor: String,
        window: TimeDelta,
        traceparent: crate::telemetry::context::TraceParent,
        reply: RpcReplyPort<Option<bool>>,
    },
}

impl crate::telemetry::context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
            Message::QueryLatest { traceparent, .. } => traceparent.as_deref(),
            Message::QuerySeenWithin { traceparent, .. } => traceparent.as_deref(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.entity.address),
            Message::QueryLatest { sensor, .. } => Some(sensor),
            Message::QuerySeenWithin { sensor, .. } => Some(sensor),
        }
    }
}

#[derive(Default)]
pub struct PresenceSensorState {
    pub last_presence: HashMap<String, bool>,
    pub sensors: HashMap<String, HashMap<String, bool>>,
    pub absent_since: HashMap<String, DateTime<Utc>>,
}

impl PresenceSensorState {
    fn seen_within(&self, sensor: &str, window: TimeDelta, now: DateTime<Utc>) -> Option<bool> {
        let present = *self.last_presence.get(sensor)?;

        let seen = present
            || self
                .absent_since
                .get(sensor)
                .is_some_and(|since| now - *since <= window);

        Some(seen)
    }
}

pub struct PresenceSensorHandler {
    shared_actor_state: AppState,
}

impl PresenceSensorHandler {
    pub const NAME: &str = "presence-sensor";

    fn process_presence(
        &self,
        event_id: Uuid,
        key: String,
        presence: bool,
        state: &mut PresenceSensorState,
    ) -> Result<(), anyhow::Error> {
        let previous = state.last_presence.insert(key.clone(), presence);
        let was_state_changed = previous != Some(presence);

        if presence {
            state.absent_since.remove(&key);
        } else if previous == Some(true) {
            state.absent_since.insert(key.clone(), Utc::now());
        }

        // edge-detected here so triggers receive discrete transitions, not every
        // sensor ping
        if was_state_changed {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Presence {
                    event_id,
                    sensor: key,
                    present: presence,
                });
        }

        Ok(())
    }

    async fn handle(
        &self,
        message: Message,
        state: &mut PresenceSensorState,
    ) -> Result<(), anyhow::Error> {
        match message {
            Message::QueryLatest { sensor, reply, .. } => {
                reply.send(state.last_presence.get(&sensor).copied())?;
            }
            Message::QuerySeenWithin {
                sensor,
                window,
                reply,
                ..
            } => {
                reply.send(state.seen_within(&sensor, window, Utc::now()))?;
            }
            Message::NewEvent(event) => {
                let Entity {
                    address,
                    sensor,
                    presence,
                } = event.entity;

                let present = match sensor {
                    Some(sensor) => {
                        let sensors = state.sensors.entry(address.clone()).or_default();
                        sensors.insert(sensor, presence);

                        sensors.values().any(|&on| on)
                    }
                    None => presence,
                };

                self.process_presence(event.event_id, address, present, state)?
            }
        }

        Ok(())
    }
}

impl DeviceHandler for PresenceSensorHandler {
    const NAME: &'static str = PresenceSensorHandler::NAME;

    type Message = Message;
    type State = PresenceSensorState;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    const ROLE: crate::decoding::DeviceRoleName = crate::decoding::DeviceRoleName::Presence;

    fn init_state(&self) -> anyhow::Result<Self::State> {
        Ok(PresenceSensorState::default())
    }

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message, state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: TimeDelta = TimeDelta::minutes(25);

    fn state(
        present: bool,
        absent_for: Option<TimeDelta>,
        now: DateTime<Utc>,
    ) -> PresenceSensorState {
        let mut state = PresenceSensorState::default();
        state.last_presence.insert("sensor".to_owned(), present);

        if let Some(absent_for) = absent_for {
            state
                .absent_since
                .insert("sensor".to_owned(), now - absent_for);
        }

        state
    }

    #[test]
    fn present_sensor_is_seen() {
        let now = Utc::now();

        assert_eq!(
            state(true, None, now).seen_within("sensor", WINDOW, now),
            Some(true)
        );
    }

    #[test]
    fn recently_vacated_sensor_is_seen() {
        let now = Utc::now();
        let state = state(false, Some(TimeDelta::minutes(1)), now);

        assert_eq!(state.seen_within("sensor", WINDOW, now), Some(true));
    }

    #[test]
    fn long_vacated_sensor_is_not_seen() {
        let now = Utc::now();
        let state = state(false, Some(TimeDelta::minutes(30)), now);

        assert_eq!(state.seen_within("sensor", WINDOW, now), Some(false));
    }

    #[test]
    fn sensor_never_present_is_not_seen() {
        let now = Utc::now();

        assert_eq!(
            state(false, None, now).seen_within("sensor", WINDOW, now),
            Some(false)
        );
    }

    #[test]
    fn unknown_sensor_has_no_answer() {
        let now = Utc::now();

        assert_eq!(
            PresenceSensorState::default().seen_within("sensor", WINDOW, now),
            None
        );
    }
}
