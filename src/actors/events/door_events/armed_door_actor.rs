use super::DoorEvents;
use crate::settings::{ArmedDoorStates, DoorSettings, IEEEAddress};
use chrono::{DateTime, Utc};
use ractor::Actor;
use std::collections::HashMap;

pub enum DoorState {
    Open,
    Closed,
}

pub struct ArmedDoorState {
    pub map: HashMap<IEEEAddress, DoorState>,
    pub last_trigger: HashMap<IEEEAddress, DateTime<Utc>>,
}

pub struct ArmedDoor {
    pub door_settings: HashMap<IEEEAddress, DoorSettings>,
}
impl ArmedDoor {
    pub const NAME: &str = "armed-door";
}

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

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            DoorEvents::Opened { ieee_addr } => {
                state.map.insert(ieee_addr.clone(), DoorState::Open);
                if let Some(value) = self.door_settings.get(&ieee_addr) {
                    if let ArmedDoorStates::Armed { timeout } = value.armed {
                        let duration = timeout.to_std()?;
                        myself.send_after(duration, || DoorEvents::Trigger { ieee_addr });
                    }
                }
            }
            DoorEvents::Closed { ieee_addr } => {
                state.map.insert(ieee_addr, DoorState::Closed);
            }
            DoorEvents::Trigger { ieee_addr } => {
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
                                    tracing::warn!("DOOR OPEN, REPLACE WITH DISCORD ACTOR")
                                }
                            } else {
                                tracing::warn!("DOOR OPEN, REPLACE WITH DISCORD ACTOR")
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
