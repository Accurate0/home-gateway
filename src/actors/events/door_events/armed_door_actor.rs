use super::{DoorEvents, DoorEventsType};
use crate::{
    notify::notify,
    settings::{ArmedDoorStates, IEEEAddress},
    types::SharedActorState,
};
use chrono::{DateTime, Utc};
use ractor::Actor;
use std::collections::HashMap;
use tracing::Level;

pub enum DoorState {
    Open,
    Closed,
}

pub struct ArmedDoorState {
    pub map: HashMap<IEEEAddress, DoorState>,
    pub last_trigger: HashMap<IEEEAddress, DateTime<Utc>>,
}

pub struct ArmedDoor {
    pub shared_actor_state: SharedActorState,
}

impl ArmedDoor {
    pub const NAME: &str = "armed-door";
}

impl ArmedDoor {
    pub fn trigger_action(&self, ieee_addr: &IEEEAddress) {
        let settings = self.shared_actor_state.settings.load();
        if let Some(settings) = settings.doors.get(ieee_addr) {
            let message = format!("{} has been left open.", settings.name);
            notify(&settings.notify, message, true);
        }
    }
}

// TODO: audit log
impl Actor for ArmedDoor {
    type Msg = DoorEvents;
    type State = ArmedDoorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(ArmedDoorState {
            map: Default::default(),
            last_trigger: Default::default(),
        })
    }

    #[tracing::instrument(name = "armed-door-actor", skip(self, myself, message, state), level = Level::TRACE)]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let DoorEvents {
            event_id,
            ieee_addr,
            ref event,
        } = message;

        let settings = self.shared_actor_state.settings.load();

        match event {
            DoorEventsType::Opened => {
                state.map.insert(ieee_addr.clone(), DoorState::Open);
                if let Some(value) = settings.doors.get(&ieee_addr)
                    && let ArmedDoorStates::Armed { timeout } = value.armed {
                        let duration = timeout.to_std()?;
                        myself.send_after(duration, move || DoorEvents {
                            ieee_addr,
                            event_id,
                            event: DoorEventsType::Trigger,
                        });
                    }
            }
            DoorEventsType::Closed => {
                state.map.insert(ieee_addr, DoorState::Closed);
            }
            DoorEventsType::Trigger => {
                let door_state = state.map.get(&ieee_addr);
                match door_state {
                    Some(door_state) => match door_state {
                        DoorState::Open => {
                            let now = chrono::offset::Utc::now();
                            if let Some(last_trigger) = state.last_trigger.get(&ieee_addr) {
                                let difference = now - last_trigger;
                                if difference.num_seconds() <= 60 {
                                    tracing::info!("de-duped event for door trigger: {ieee_addr}");
                                } else {
                                    self.trigger_action(&ieee_addr);
                                }
                            } else {
                                self.trigger_action(&ieee_addr);
                            }

                            state.last_trigger.insert(ieee_addr, now);
                        }
                        // do nothing door has been closed since event
                        DoorState::Closed => {}
                    },
                    None => tracing::warn!("can't check door state, does not exist"),
                }
            }
        }

        Ok(())
    }
}
