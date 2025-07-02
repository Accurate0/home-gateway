use crate::{
    settings::{ApplianceSettings, IEEEAddress},
    types::SharedActorState,
};
use appliance_state_actor::ApplianceState;
use ractor::Actor;
use std::collections::HashMap;
use uuid::Uuid;

mod appliance_state_actor;

pub enum ApplianceEvents {
    #[allow(unused)]
    PowerUsage {
        event_id: Uuid,
        ieee_addr: String,
        power: i64,
        energy: f64,
        voltage: i64,
        current: f64,
    },
}

pub struct ApplianceEventsSupervisor {
    pub shared_actor_state: SharedActorState,
    pub appliance_settings: HashMap<IEEEAddress, ApplianceSettings>,
}

impl ApplianceEventsSupervisor {
    pub const NAME: &str = "appliance-supervisor";
    pub const GROUP_NAME: &str = "appliance-events";

    async fn start_appliance_state_actor(
        appliance_settings: HashMap<IEEEAddress, ApplianceSettings>,
        shared_actor_state: SharedActorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let (armed_door_actor, _) = Actor::spawn(
            Some(ApplianceState::NAME.to_string()),
            ApplianceState {
                appliance_settings,
                shared_actor_state,
            },
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

impl Actor for ApplianceEventsSupervisor {
    type Msg = ();
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Self::start_appliance_state_actor(
            self.appliance_settings.clone(),
            self.shared_actor_state.clone(),
        )
        .await
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
                        ApplianceState::NAME => {
                            Self::start_appliance_state_actor(
                                self.appliance_settings.clone(),
                                self.shared_actor_state.clone(),
                            )
                            .await?
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
