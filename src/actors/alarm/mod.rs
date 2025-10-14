use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    settings::workflow::{
        WorkflowEntityLightQueryState, WorkflowEntityLightTypeState, WorkflowEntityType,
        WorkflowQueryType, WorkflowSettings,
    },
    types::SharedActorState,
};
use chrono::{DateTime, TimeDelta, Utc};
use ractor::{
    Actor,
    factory::{FactoryMessage, Job, JobOptions},
};
use std::time::Duration;
use types::AndroidAppAlarmPayload;
use uuid::Uuid;

pub mod types;

pub enum AlarmMessage {
    NextAlarm(AndroidAppAlarmPayload),
    CheckIfAlarmWillTrigger { offset: TimeDelta },
}

pub struct AlarmActor {
    pub shared_actor_state: SharedActorState,
}

impl AlarmActor {
    pub const NAME: &str = "alarm";
    const ALARM_STATE_KEY: &str = "next_alarm";
    const LAMP_IEEE_ADDR: &str = "0x001788010381ef5d";
}

impl Actor for AlarmActor {
    type Msg = AlarmMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let _join_handle = myself.send_interval(Duration::from_secs(60), || {
            AlarmMessage::CheckIfAlarmWillTrigger {
                offset: TimeDelta::minutes(5),
            }
        });

        Ok(())
    }

    #[tracing::instrument(name = "alarm-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            AlarmMessage::NextAlarm(android_app_alarm_payload) => {
                tracing::info!("alarm local: {}", android_app_alarm_payload.local_time);
                sqlx::query!(
                    "INSERT INTO state (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
                    Self::ALARM_STATE_KEY,
                    android_app_alarm_payload.local_time
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }
            AlarmMessage::CheckIfAlarmWillTrigger { offset } => {
                let maybe_alarm_time = sqlx::query!(
                    "SELECT value FROM state WHERE key = $1",
                    Self::ALARM_STATE_KEY
                )
                .fetch_optional(&self.shared_actor_state.db)
                .await?;

                if let Some(alarm_time) = maybe_alarm_time {
                    let now = Utc::now();
                    let alarm_time = DateTime::parse_from_rfc3339(&alarm_time.value)?;

                    let should_trigger =
                        now.timestamp_millis() >= (alarm_time - offset).timestamp_millis();

                    if should_trigger {
                        tracing::info!("triggering workflow to turn on lamp");
                        let Some(workflow_actor) =
                            ractor::registry::where_is(WorkflowWorker::NAME.to_owned())
                        else {
                            tracing::warn!("could not find workflow actor");
                            return Ok(());
                        };

                        let workflow = WorkflowSettings {
                            enabled: true,
                            run: vec![WorkflowEntityType::Conditional {
                                run: vec![
                                    WorkflowEntityType::Light {
                                        ieee_addr: Self::LAMP_IEEE_ADDR.to_owned(),
                                        state: WorkflowEntityLightTypeState::SetBrightness {
                                            value: 1,
                                        },
                                        when: None,
                                    },
                                    WorkflowEntityType::Light {
                                        ieee_addr: Self::LAMP_IEEE_ADDR.to_owned(),
                                        state: WorkflowEntityLightTypeState::IncreaseBrightness {
                                            value: 1,
                                            on_off: false,
                                        },
                                        when: None,
                                    },
                                ],
                                when: WorkflowQueryType::Light {
                                    ieee_addr: Self::LAMP_IEEE_ADDR.to_owned(),
                                    state: WorkflowEntityLightQueryState::Off,
                                },
                            }],
                        };

                        let event_id = Uuid::new_v4();
                        let message = FactoryMessage::Dispatch(Job {
                            key: (),
                            msg: WorkflowWorkerMessage::Execute { event_id, workflow },
                            options: JobOptions::default(),
                            accepted: None,
                        });

                        workflow_actor.send_message(message)?;
                        sqlx::query!("DELETE FROM state WHERE key = $1", Self::ALARM_STATE_KEY)
                            .execute(&self.shared_actor_state.db)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
