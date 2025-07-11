use crate::{delayqueue::DelayQueue, notify::notify, settings::NotifySource};
use ractor::Actor;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod background;

pub enum ReminderActorMessage {
    SetReminder {
        message: String,
        delay: Duration,
        channel_id: u64,
        user_id: u64,
    },
    TriggerReminder {
        message: String,
        channel_id: u64,
        user_id: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReminderActorDelayQueueValue {
    message: String,
    delay: Duration,
    channel_id: u64,
    user_id: u64,
}

pub struct ReminderActor {
    pub delay_queue: DelayQueue<ReminderActorDelayQueueValue>,
}

impl ReminderActor {
    pub const NAME: &str = "reminder";
    pub const QUEUE_NAME: &str = "reminder";
}

impl Actor for ReminderActor {
    type Msg = ReminderActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ReminderActorMessage::SetReminder {
                message,
                delay,
                channel_id,
                user_id,
            } => {
                let v = ReminderActorDelayQueueValue {
                    message,
                    delay,
                    channel_id,
                    user_id,
                };

                self.delay_queue.push(v, delay).await?;
            }
            ReminderActorMessage::TriggerReminder {
                message,
                channel_id,
                user_id,
            } => {
                let notify_source = NotifySource::Discord {
                    channel_id,
                    mentions: vec![user_id],
                };

                let message = format!("Reminder about \"{}\"", message);
                notify(&[notify_source], message);
            }
        }

        Ok(())
    }
}
