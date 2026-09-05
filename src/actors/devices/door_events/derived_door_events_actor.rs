use super::{DoorEvents, DoorEventsMessage, DoorEventsType};
use crate::db::DoorState;
use crate::event_bus::EventBusMessage;
use crate::repo::door::DerivedDoorEvent;
use crate::{
    settings::{DoorSettings, IEEEAddress},
    state::AppState,
};
use chrono::{DateTime, Utc};
use ractor::Actor;
use std::collections::HashMap;

pub struct DerivedDoorEventsState {
    pub map: HashMap<IEEEAddress, DoorState>,
    pub last_trigger: HashMap<IEEEAddress, DateTime<Utc>>,
}

pub struct DerivedDoorEvents {
    pub shared_actor_state: AppState,
}

impl DerivedDoorEvents {
    pub const NAME: &str = "derived-door-events";

    async fn change_door_state(
        &self,
        message: &DoorEvents,
        state: &mut DerivedDoorEventsState,
        now: DateTime<Utc>,
        door_settings: &DoorSettings,
        door_state: DoorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        self.shared_actor_state
            .repos
            .door()
            .append_derived(&DerivedDoorEvent {
                event_id: message.event_id,
                name: door_settings.name.clone(),
                id: door_settings.id.clone(),
                ieee_addr: message.ieee_addr.clone(),
                state: door_state,
            })
            .await?;

        state.map.insert(message.ieee_addr.clone(), door_state);

        state.last_trigger.insert(message.ieee_addr.clone(), now);

        // publish the confirmed, debounced transition so `door` triggers can fire
        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Door {
                event_id: message.event_id,
                ieee_addr: message.ieee_addr.clone(),
                open: matches!(door_state, DoorState::Open),
            });

        Ok(())
    }
}

impl Actor for DerivedDoorEvents {
    type Msg = DoorEventsMessage;
    type State = DerivedDoorEventsState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let map = self
            .shared_actor_state
            .repos
            .door()
            .latest_derived_per_door()
            .await?;

        Ok(DerivedDoorEventsState {
            map,
            last_trigger: Default::default(),
        })
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let message = match message {
            DoorEventsMessage::Event(event) => event,
            DoorEventsMessage::QueryState { ieee_addr, reply } => {
                reply.send(state.map.get(&ieee_addr).copied())?;
                return Ok(());
            }
        };

        if let Some(door_settings) = self.shared_actor_state.devices.door(&message.ieee_addr) {
            let last_state = state.map.get(&message.ieee_addr);
            let now = chrono::offset::Utc::now();
            let last_event_is_too_soon = state
                .last_trigger
                .get(&message.ieee_addr)
                .map(|d| {
                    let difference = now - d;
                    difference.as_seconds_f64() < 1.0
                })
                .unwrap_or(false);

            if last_event_is_too_soon {
                return Ok(());
            }

            match message.event {
                DoorEventsType::Opened => match last_state {
                    Some(last_state) => match last_state {
                        DoorState::Open => {
                            // do nothing
                        }
                        DoorState::Closed => {
                            self.change_door_state(
                                &message,
                                state,
                                now,
                                door_settings,
                                DoorState::Open,
                            )
                            .await?;
                        }
                    },
                    None => {
                        self.change_door_state(
                            &message,
                            state,
                            now,
                            door_settings,
                            DoorState::Open,
                        )
                        .await?;
                    }
                },
                DoorEventsType::Closed => match last_state {
                    Some(door_state) => match door_state {
                        DoorState::Open => {
                            self.change_door_state(
                                &message,
                                state,
                                now,
                                door_settings,
                                DoorState::Closed,
                            )
                            .await?;
                        }
                        DoorState::Closed => {
                            // do nothing
                        }
                    },
                    None => {
                        self.change_door_state(
                            &message,
                            state,
                            now,
                            door_settings,
                            DoorState::Closed,
                        )
                        .await?
                    }
                },
                DoorEventsType::Trigger => {}
            }
        };

        Ok(())
    }
}
