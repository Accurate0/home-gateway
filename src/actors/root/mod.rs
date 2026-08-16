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
use std::collections::HashMap;
use std::time::Duration;

use super::{
    alarm::AlarmActor,
    devices::{
        control_switch::{self, ControlSwitchHandler},
        door_events::DoorEventsSupervisor,
        door_sensor::{self, DoorSensorHandler},
        environment_sensor::{self, EnvironmentSensorHandler},
        light::{self, LightHandler},
        media_player::MediaPlayerHandler,
        plant_sensor::{self, PlantSensorHandler},
        presence_sensor::{self, PresenceSensorHandler},
        robot_vacuum::RobotVacuumHandler,
        smart_switch::{self, SmartSwitchHandler},
    },
    integrations::{
        home_assistant::HomeAssistantActor, jellyfin::JellyfinActor, synergy::SynergyActor,
        unifi::UnifiConnectedClientHandler,
    },
    system::{
        cron::CronActor,
        mqtt_ingest::{self, MqttIngest},
        push::{self, PushWorker},
    },
    workflows::{self, WorkflowWorker, dispatcher::WorkflowDispatcher},
};

const RESTART_BACKOFF_BASE: Duration = Duration::from_secs(1);
const RESTART_BACKOFF_MAX: Duration = Duration::from_secs(60);
const RESTART_BACKOFF_MAX_SHIFT: u32 = 16;

pub enum RootMessage {
    Restart(String),
}

#[derive(Default)]
pub struct RootState {
    attempts: HashMap<String, u32>,
}

pub struct RootSupervisor {
    pub shared_actor_state: SharedActorState,
}

impl RootSupervisor {
    fn schedule_restart(
        &self,
        myself: &ractor::ActorRef<RootMessage>,
        state: &mut RootState,
        name: String,
        delay: Duration,
    ) {
        state.attempts.entry(name.clone()).or_insert(0);
        myself.send_after(delay, move || RootMessage::Restart(name));
    }

    async fn restart(
        &self,
        myself: &ractor::ActorRef<RootMessage>,
        name: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        tracing::info!(actor = %name, "restarting actor");

        let state = self.shared_actor_state.clone();

        match name {
            UnifiConnectedClientHandler::NAME => {
                self.start_unifi_connected_clients_handler(myself).await?
            }
            CronActor::NAME => self.start_cron_actor(myself).await?,
            SynergyActor::NAME => self.start_synergy_actor(myself).await?,
            WoolworthsActor::NAME => self.start_woolworths_actor(myself).await?,
            TrmnlActor::NAME => self.start_trmnl_actor(myself).await?,
            AlarmActor::NAME => self.start_alarm_actor(myself).await?,
            EInkDisplayActor::NAME => self.start_eink_display_actor(myself).await?,
            BatteryActor::NAME => self.start_battery_actor(myself).await?,
            HomeAssistantActor::NAME => self.start_home_assistant_actor(myself).await?,
            JellyfinActor::NAME => self.start_jellyfin_actor(myself).await?,
            SolarActor::NAME => self.start_solar_actor(myself).await?,
            SunActor::NAME => self.start_sun_actor(myself).await?,
            WatchdogActor::NAME => self.start_watchdog_actor(myself).await?,
            WorkflowDispatcher::NAME => self.start_workflow_dispatcher(myself).await?,

            MqttIngest::NAME => {
                mqtt_ingest::spawn::spawn_mqtt_ingest(myself, state).await?;
            }
            WorkflowWorker::NAME => {
                workflows::spawn::spawn_workflows(myself, state).await?;
            }
            DoorEventsSupervisor::NAME => {
                myself
                    .spawn_linked(
                        Some(DoorEventsSupervisor::NAME.to_string()),
                        DoorEventsSupervisor {
                            shared_actor_state: state,
                        },
                        (),
                    )
                    .await?;
            }
            LightHandler::NAME => {
                light::spawn::spawn_light_handler(myself, state).await?;
            }
            PresenceSensorHandler::NAME => {
                presence_sensor::spawn::spawn_presence_handler(myself, state).await?;
            }
            EnvironmentSensorHandler::NAME => {
                environment_sensor::spawn::spawn_environment_sensor_handler(myself, state).await?;
            }
            PlantSensorHandler::NAME => {
                plant_sensor::spawn::spawn_plant_sensor_handler(myself, state).await?;
            }
            DoorSensorHandler::NAME => {
                door_sensor::spawn::spawn_door_handler(myself, state).await?;
            }
            ControlSwitchHandler::NAME => {
                control_switch::spawn::spawn_control_switch_handler(myself, state).await?;
            }
            SmartSwitchHandler::NAME => {
                smart_switch::spawn::spawn_smart_switch_handler(myself, state).await?;
            }
            MediaPlayerHandler::NAME => {
                crate::actors::devices::media_player::spawn::spawn_media_player_handler(
                    myself, state,
                )
                .await?;
            }
            RobotVacuumHandler::NAME => {
                crate::actors::devices::robot_vacuum::spawn::spawn_robot_vacuum_handler(
                    myself, state,
                )
                .await?;
            }
            PushWorker::NAME => {
                push::spawn::spawn_push(myself, state).await?;
            }

            name => tracing::error!(actor = %name, "no restart hook for this actor"),
        }

        Ok(())
    }

    async fn start_eink_display_actor(
        &self,
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
        myself: &ractor::ActorRef<RootMessage>,
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
    type Msg = RootMessage;
    type State = RootState;
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

        Ok(RootState::default())
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        state: &mut Self::State,
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

                self.schedule_restart(&myself, state, name.to_string(), Duration::ZERO);
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            RootMessage::Restart(name) => {
                if let Err(e) = self.restart(&myself, &name).await {
                    let attempts = state.attempts.entry(name.clone()).or_insert(0);
                    *attempts += 1;

                    let backoff = restart_backoff(*attempts);

                    tracing::error!(
                        actor = %name,
                        attempts = *attempts,
                        ?backoff,
                        "failed to restart actor, retrying: {e}"
                    );

                    myself.send_after(backoff, move || RootMessage::Restart(name));

                    return Ok(());
                }

                state.attempts.remove(&name);
            }
        }

        Ok(())
    }
}

fn restart_backoff(attempts: u32) -> Duration {
    RESTART_BACKOFF_BASE
        .saturating_mul(
            2u32.saturating_pow(attempts.saturating_sub(1).min(RESTART_BACKOFF_MAX_SHIFT)),
        )
        .min(RESTART_BACKOFF_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_backoff_grows_and_saturates() {
        assert_eq!(restart_backoff(1), RESTART_BACKOFF_BASE);
        assert_eq!(restart_backoff(2), RESTART_BACKOFF_BASE * 2);
        assert_eq!(restart_backoff(3), RESTART_BACKOFF_BASE * 4);
        assert_eq!(restart_backoff(u32::MAX), RESTART_BACKOFF_MAX);
    }

    #[test]
    fn restart_backoff_never_exceeds_the_cap() {
        for attempts in 1..64 {
            assert!(restart_backoff(attempts) <= RESTART_BACKOFF_MAX);
        }
    }
}
