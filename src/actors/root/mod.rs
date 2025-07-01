use crate::{settings::Settings, types::SharedActorState};
use ractor::Actor;

use super::{
    devices::unifi::UnifiConnectedClientHandler,
    door_sensor,
    events::{appliances::ApplianceEventsSupervisor, door_events::DoorEventsSupervisor},
    light, smart_switch, temperature_sensor,
};

pub struct RootSupervisor {
    pub shared_actor_state: SharedActorState,
    pub settings: Settings,
}

impl RootSupervisor {
    async fn start_unifi_connected_clients_handler(
        &self,
        myself: ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(UnifiConnectedClientHandler::NAME.to_owned()),
                UnifiConnectedClientHandler {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }
}

impl Actor for RootSupervisor {
    type Msg = ();
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let shared_actor_state = &self.shared_actor_state;
        let settings = &self.settings;

        smart_switch::spawn::spawn_smart_switch_handler(&myself, shared_actor_state.clone())
            .await?;

        door_sensor::spawn::spawn_door_handler(&myself, shared_actor_state.clone()).await?;

        light::spawn::spawn_light_handler(&myself, shared_actor_state.clone()).await?;
        temperature_sensor::spawn::spawn_temperature_sensor_handler(
            &myself,
            shared_actor_state.clone(),
        )
        .await?;

        myself
            .spawn_linked(
                Some(DoorEventsSupervisor::NAME.to_string()),
                DoorEventsSupervisor {
                    door_settings: settings.doors.clone(),
                },
                (),
            )
            .await?;

        myself
            .spawn_linked(
                Some(ApplianceEventsSupervisor::NAME.to_string()),
                ApplianceEventsSupervisor {
                    shared_actor_state: shared_actor_state.clone(),
                    appliance_settings: settings.appliances.clone(),
                },
                (),
            )
            .await?;

        self.start_unifi_connected_clients_handler(myself).await?;

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match &message {
            ractor::SupervisionEvent::ActorTerminated(who, _, _)
            | ractor::SupervisionEvent::ActorFailed(who, _) => {
                tracing::error!("actor {who:?} failed");
                if let ractor::SupervisionEvent::ActorFailed(_, panic) = &message {
                    tracing::error!("panic: {panic}");
                }

                if who
                    .get_name()
                    .is_some_and(|n| n == UnifiConnectedClientHandler::NAME)
                {
                    tracing::info!("restarting unifi handler");
                    self.start_unifi_connected_clients_handler(myself).await?;
                };
            }
            _ => {}
        }
        Ok(())
    }
}
