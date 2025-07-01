use crate::settings::{DoorSettings, IEEEAddress};
use armed_door_actor::ArmedDoor;
use ractor::Actor;
use std::collections::HashMap;

mod armed_door_actor;

pub enum DoorEvents {
    Opened { ieee_addr: String },
    Closed { ieee_addr: String },
    Trigger { ieee_addr: String },
}

pub struct DoorEventsSupervisor {
    pub door_settings: HashMap<IEEEAddress, DoorSettings>,
}

impl DoorEventsSupervisor {
    pub const NAME: &str = "door-events-supervisor";
    pub const GROUP_NAME: &str = "door-events";

    async fn start_armed_door_actor(
        door_settings: HashMap<IEEEAddress, DoorSettings>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let (armed_door_actor, _) = Actor::spawn(
            Some(ArmedDoor::NAME.to_string()),
            ArmedDoor { door_settings },
            (),
        )
        .await?;

        ractor::pg::join(
            Self::GROUP_NAME.to_string(),
            vec![armed_door_actor.get_cell()],
        );

        Ok(())
    }
}

impl Actor for DoorEventsSupervisor {
    type Msg = ();
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Self::start_armed_door_actor(self.door_settings.clone()).await
    }

    async fn handle_supervisor_evt(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ractor::SupervisionEvent::ActorTerminated(who, _, _)
            | ractor::SupervisionEvent::ActorFailed(who, _) => {
                tracing::error!("actor: {who:?} failed, restarting");
                if let Some(e) = who.get_name() {
                    match e.as_str() {
                        ArmedDoor::NAME => {
                            Self::start_armed_door_actor(self.door_settings.clone()).await?
                        }
                        actor => tracing::warn!("unknown: {actor}, cannot restart"),
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
