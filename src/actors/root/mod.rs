use crate::{delayqueue::DelayQueue, settings::Settings, types::SharedActorState};
use ractor::Actor;

use super::{
    devices::unifi::UnifiConnectedClientHandler,
    door_sensor,
    events::{appliances::ApplianceEventsSupervisor, door_events::DoorEventsSupervisor},
    light,
    maccas::MaccasActor,
    reminder::{ReminderActor, ReminderActorDelayQueueValue},
    selfbot, smart_switch, temperature_sensor,
};

pub struct RootSupervisor {
    pub shared_actor_state: SharedActorState,
    pub settings: Settings,
    pub reminder_delayqueue: DelayQueue<ReminderActorDelayQueueValue>,
}

impl RootSupervisor {
    async fn start_unifi_connected_clients_handler(
        &self,
        myself: &ractor::ActorRef<()>,
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

    async fn start_reminder_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(ReminderActor::NAME.to_owned()),
                ReminderActor {
                    delay_queue: self.reminder_delayqueue.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_maccas_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(MaccasActor::NAME.to_owned()),
                MaccasActor {
                    settings: self.settings.maccas.clone(),
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
        selfbot::spawn::spawn_selfbot(&myself, settings.clone()).await?;

        light::spawn::spawn_light_handler(&myself, shared_actor_state.clone()).await?;
        temperature_sensor::spawn::spawn_temperature_sensor_handler(
            &myself,
            shared_actor_state.clone(),
            settings.clone(),
        )
        .await?;

        myself
            .spawn_linked(
                Some(DoorEventsSupervisor::NAME.to_string()),
                DoorEventsSupervisor {
                    shared_actor_state: shared_actor_state.clone(),
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

        self.start_maccas_actor(&myself).await?;
        self.start_unifi_connected_clients_handler(&myself).await?;
        self.start_reminder_actor(&myself).await?;

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
                    self.start_unifi_connected_clients_handler(&myself).await?;
                };

                if who.get_name().is_some_and(|n| n == MaccasActor::NAME) {
                    tracing::info!("restarting maccas actor");
                    self.start_maccas_actor(&myself).await?;
                };

                if who.get_name().is_some_and(|n| n == ReminderActor::NAME) {
                    tracing::info!("restarting reminder actor");
                    self.start_reminder_actor(&myself).await?;
                };
            }
            _ => {}
        }
        Ok(())
    }
}
