use crate::{
    delayqueue::DelayQueue,
    notify::notify,
    settings::{NotifySource, ReminderSettings, ReminderState},
    types::SharedActorState,
};
use ractor::Actor;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod background;
pub mod cronlike_expression;

pub enum ReminderActorMessage {
    SetReminder {
        message: String,
        delay: Duration,
        channel_id: u64,
        user_id: u64,
    },

    TriggerScheduledReminder {
        message: String,
        notify: Vec<NotifySource>,
        scheduled_reminder: ReminderSettings,
    },
    TriggerReminder {
        message: String,
        channel_id: u64,
        user_id: Vec<u64>,
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
    pub shared_actor_state: SharedActorState,
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
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        for reminder in &self.shared_actor_state.settings.reminders {
            let time = reminder.frequency.next_trigger(reminder.starts_on).await;
            let reminder = reminder.clone();

            myself.send_after(Duration::from_secs_f64(time.as_seconds_f64()), move || {
                ReminderActorMessage::TriggerScheduledReminder {
                    message: format!("Scheduled reminder about \"{}\"", reminder.name),
                    notify: reminder.notify.clone(),
                    scheduled_reminder: reminder,
                }
            });
        }

        Ok(())
    }

    #[tracing::instrument(name = "reminder-actor", skip(self, myself, message, _state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
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
                    mentions: user_id,
                };

                let message = format!("Reminder about \"{}\"", message);
                notify(&[notify_source], message, true);
            }
            ReminderActorMessage::TriggerScheduledReminder {
                message,
                notify: notify_sources,
                scheduled_reminder,
            } => {
                tracing::info!(
                    "run {} for {scheduled_reminder:?}",
                    scheduled_reminder.state
                );

                if scheduled_reminder.state == ReminderState::Active {
                    notify(&notify_sources, message, true);
                }

                let time = scheduled_reminder
                    .frequency
                    .next_trigger(scheduled_reminder.starts_on)
                    .await;

                let reminder = scheduled_reminder.clone();
                myself.send_after(Duration::from_secs_f64(time.as_seconds_f64()), move || {
                    ReminderActorMessage::TriggerScheduledReminder {
                        message: format!("Scheduled reminder about \"{}\"", reminder.name),
                        notify: reminder.notify.clone(),
                        scheduled_reminder: reminder,
                    }
                });
            }
        }

        Ok(())
    }
}
