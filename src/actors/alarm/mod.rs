use crate::types::SharedActorState;
use ractor::Actor;
use types::AndroidAppAlarmPayload;

pub mod types;

pub enum AlarmMessage {
    NextAlarm(AndroidAppAlarmPayload),
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
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
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
                    "INSERT INTO state (key, value) VALUES ($1, $2)",
                    Self::ALARM_STATE_KEY,
                    android_app_alarm_payload.local_time
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }
        }

        Ok(())
    }
}
