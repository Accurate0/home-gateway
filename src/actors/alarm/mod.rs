use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    types::SharedActorState,
};
use chrono::{DateTime, TimeDelta, Utc};
use ractor::{
    Actor,
    factory::{FactoryMessage, Job, JobOptions},
};
use std::time::Duration;
use tracing::Level;
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
        let offset = self.shared_actor_state.settings.alarm.offset;
        let _join_handle = myself.send_interval(Duration::from_secs(60), move || {
            AlarmMessage::CheckIfAlarmWillTrigger { offset }
        });

        Ok(())
    }

    #[tracing::instrument(name = "alarm-actor", skip(self, _myself, message, _state), level = Level::TRACE)]
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
                        let workflow_name = &self.shared_actor_state.settings.alarm.workflow;
                        tracing::info!("triggering `{workflow_name}` workflow");
                        let Some(workflow_actor) =
                            ractor::registry::where_is(WorkflowWorker::NAME.to_owned())
                        else {
                            tracing::warn!("could not find workflow actor");
                            return Ok(());
                        };

                        let Some(workflow) =
                            self.shared_actor_state.settings.workflows.get(workflow_name)
                        else {
                            tracing::warn!("alarm workflow `{workflow_name}` not configured");
                            return Ok(());
                        };
                        let workflow = workflow.clone();

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
