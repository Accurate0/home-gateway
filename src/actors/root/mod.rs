use crate::{
    actors::{
        eink_display::EInkDisplayActor,
        integrations::{solar::SolarActor, trmnl::TrmnlActor, woolworths::WoolworthsActor},
        sun::SunActor,
        system::{battery::BatteryActor, watchdog::WatchdogActor},
    },
    integrations::{
        solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI},
        trmnl::Trmnl,
        woolworths::Woolworths,
    },
    state::SharedActorState,
};
use ractor::Actor;

use super::{
    alarm::AlarmActor,
    devices::{
        control_switch, door_events::DoorEventsSupervisor, door_sensor, environment_sensor, light,
        plant_sensor, presence_sensor, smart_switch,
    },
    integrations::{
        home_assistant::HomeAssistantActor, jellyfin::JellyfinActor, synergy::SynergyActor,
        unifi::UnifiConnectedClientHandler,
    },
    system::{cron::CronActor, push},
    workflows::{self, dispatcher::WorkflowDispatcher},
};

pub struct RootSupervisor {
    pub shared_actor_state: SharedActorState,
}

impl RootSupervisor {
    async fn start_eink_display_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(EInkDisplayActor::NAME.to_owned()),
                EInkDisplayActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_battery_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(BatteryActor::NAME.to_owned()),
                BatteryActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_home_assistant_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        if self.shared_actor_state.home_assistant.is_none() {
            return Ok(());
        }

        myself
            .spawn_linked(
                Some(HomeAssistantActor::NAME.to_owned()),
                HomeAssistantActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_jellyfin_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(jellyfin) = self.shared_actor_state.jellyfin.clone() else {
            return Ok(());
        };

        myself
            .spawn_linked(
                Some(JellyfinActor::NAME.to_owned()),
                JellyfinActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                    jellyfin,
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_alarm_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(AlarmActor::NAME.to_owned()),
                AlarmActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

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

    async fn start_cron_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(CronActor::NAME.to_owned()),
                CronActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_woolworths_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(WoolworthsActor::NAME.to_owned()),
                WoolworthsActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                    woolworths: Woolworths::new(self.shared_actor_state.db.clone()),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_trmnl_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(api_key) = self.shared_actor_state.settings.trmnl_api_key.clone() else {
            return Ok(());
        };

        myself
            .spawn_linked(
                Some(TrmnlActor::NAME.to_owned()),
                TrmnlActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                    trmnl: Trmnl::new(
                        api_key,
                        self.shared_actor_state.settings.trmnl.base_url.clone(),
                    ),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_synergy_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(SynergyActor::NAME.to_owned()),
                SynergyActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_solar_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(settings) = self.shared_actor_state.settings.solar.clone() else {
            tracing::info!("no solar config; skipping solar actor");
            return Ok(());
        };

        let Some(goodwe) = GoodWeSemsAPI::new(self.shared_actor_state.db.clone(), &settings) else {
            return Ok(());
        };

        myself
            .spawn_linked(
                Some(SolarActor::NAME.to_owned()),
                SolarActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                    goodwe,
                    weather: WeatherAPI::new()?,
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_sun_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(SunActor::NAME.to_owned()),
                SunActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_watchdog_actor(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(WatchdogActor::NAME.to_owned()),
                WatchdogActor {
                    shared_actor_state: self.shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        Ok(())
    }

    async fn start_workflow_dispatcher(
        &self,
        myself: &ractor::ActorRef<()>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        myself
            .spawn_linked(
                Some(WorkflowDispatcher::NAME.to_owned()),
                WorkflowDispatcher {
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

        workflows::spawn::spawn_workflows(&myself, shared_actor_state.clone()).await?;

        control_switch::spawn::spawn_control_switch_handler(&myself, shared_actor_state.clone())
            .await?;

        smart_switch::spawn::spawn_smart_switch_handler(&myself, shared_actor_state.clone())
            .await?;

        door_sensor::spawn::spawn_door_handler(&myself, shared_actor_state.clone()).await?;
        push::spawn::spawn_push(&myself, self.shared_actor_state.clone()).await?;

        light::spawn::spawn_light_handler(&myself, shared_actor_state.clone()).await?;
        environment_sensor::spawn::spawn_environment_sensor_handler(
            &myself,
            shared_actor_state.clone(),
        )
        .await?;

        presence_sensor::spawn::spawn_presence_handler(&myself, shared_actor_state.clone()).await?;

        plant_sensor::spawn::spawn_plant_sensor_handler(&myself, shared_actor_state.clone())
            .await?;

        crate::actors::devices::media_player::spawn::spawn_media_player_handler(
            &myself,
            shared_actor_state.clone(),
        )
        .await?;

        crate::actors::devices::robot_vacuum::spawn::spawn_robot_vacuum_handler(
            &myself,
            shared_actor_state.clone(),
        )
        .await?;

        myself
            .spawn_linked(
                Some(DoorEventsSupervisor::NAME.to_string()),
                DoorEventsSupervisor {
                    shared_actor_state: shared_actor_state.clone(),
                },
                (),
            )
            .await?;

        self.start_unifi_connected_clients_handler(&myself).await?;
        self.start_cron_actor(&myself).await?;
        self.start_synergy_actor(&myself).await?;
        self.start_woolworths_actor(&myself).await?;
        self.start_trmnl_actor(&myself).await?;
        self.start_alarm_actor(&myself).await?;
        self.start_home_assistant_actor(&myself).await?;
        self.start_jellyfin_actor(&myself).await?;
        self.start_eink_display_actor(&myself).await?;
        self.start_battery_actor(&myself).await?;
        self.start_solar_actor(&myself).await?;
        self.start_sun_actor(&myself).await?;
        self.start_watchdog_actor(&myself).await?;
        self.start_workflow_dispatcher(&myself).await?;

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

                let Some(name) = who.get_name() else {
                    tracing::error!("actor without name failed, cannot restart");
                    return Ok(());
                };

                match name.as_str() {
                    UnifiConnectedClientHandler::NAME => {
                        tracing::info!("restarting unifi handler");
                        self.start_unifi_connected_clients_handler(&myself).await?;
                    }

                    CronActor::NAME => {
                        tracing::info!("restarting cron actor");
                        self.start_cron_actor(&myself).await?;
                    }

                    SynergyActor::NAME => {
                        tracing::info!("restarting synergy actor");
                        self.start_synergy_actor(&myself).await?;
                    }

                    WoolworthsActor::NAME => {
                        tracing::info!("restarting woolworths actor");
                        self.start_woolworths_actor(&myself).await?;
                    }

                    TrmnlActor::NAME => {
                        tracing::info!("restarting trmnl actor");
                        self.start_trmnl_actor(&myself).await?;
                    }

                    AlarmActor::NAME => {
                        tracing::info!("restarting alarm actor");
                        self.start_alarm_actor(&myself).await?;
                    }

                    EInkDisplayActor::NAME => {
                        tracing::info!("restarting eink display actor");
                        self.start_eink_display_actor(&myself).await?;
                    }

                    BatteryActor::NAME => {
                        tracing::info!("restarting battery actor");
                        self.start_battery_actor(&myself).await?;
                    }

                    HomeAssistantActor::NAME => {
                        tracing::info!("restarting home assistant actor");
                        self.start_home_assistant_actor(&myself).await?;
                    }

                    JellyfinActor::NAME => {
                        tracing::info!("restarting jellyfin actor");
                        self.start_jellyfin_actor(&myself).await?;
                    }

                    SolarActor::NAME => {
                        tracing::info!("restarting solar actor");
                        self.start_solar_actor(&myself).await?;
                    }

                    SunActor::NAME => {
                        tracing::info!("restarting sun actor");
                        self.start_sun_actor(&myself).await?;
                    }

                    WatchdogActor::NAME => {
                        tracing::info!("restarting watchdog actor");
                        self.start_watchdog_actor(&myself).await?;
                    }

                    WorkflowDispatcher::NAME => {
                        tracing::info!("restarting event dispatcher");
                        self.start_workflow_dispatcher(&myself).await?;
                    }

                    name => {
                        tracing::warn!("unknown actor: {name}, no restart hook");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
