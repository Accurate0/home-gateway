use crate::{repo::door::DoorState, settings::IEEEAddress, state::AppState};
pub use derived_door_events_actor::DerivedDoorEvents;
use ractor::{Actor, ActorCell, RpcReplyPort};
use uuid::Uuid;

pub mod lua;
pub mod query;

mod derived_door_events_actor;

pub enum DoorEventsType {
    Opened,
    Closed,
}

pub struct DoorEvents {
    pub event_id: Uuid,
    pub ieee_addr: IEEEAddress,
    pub event: DoorEventsType,
}

pub enum DoorEventsMessage {
    Event(DoorEvents),
    QueryState {
        ieee_addr: IEEEAddress,
        reply: RpcReplyPort<Option<DoorState>>,
    },
}

pub struct DoorEventsSupervisor {
    pub shared_actor_state: AppState,
}

impl DoorEventsSupervisor {
    pub const NAME: &str = "door-events-supervisor";
    pub const GROUP_NAME: &str = "door-events";

    async fn start_derived_door_events_actor(
        myself: ActorCell,
        shared_actor_state: AppState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let (derived_door_events_actor, _) = Actor::spawn_linked(
            Some(DerivedDoorEvents::NAME.to_owned()),
            DerivedDoorEvents { shared_actor_state },
            (),
            myself,
        )
        .await?;

        ractor::pg::join(
            Self::GROUP_NAME.to_string(),
            vec![derived_door_events_actor.get_cell()],
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
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Self::start_derived_door_events_actor(myself.get_cell(), self.shared_actor_state.clone())
            .await
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ractor::SupervisionEvent::ActorTerminated(who, _, _)
            | ractor::SupervisionEvent::ActorFailed(who, _) => {
                tracing::error!("actor: {who:?} failed, restarting");
                if let Some(e) = who.get_name() {
                    match e.as_str() {
                        DerivedDoorEvents::NAME => {
                            Self::start_derived_door_events_actor(
                                myself.get_cell(),
                                self.shared_actor_state.clone(),
                            )
                            .await?;
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
