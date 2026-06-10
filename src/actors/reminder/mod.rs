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
    Set {
        message: String,
        delay: Duration,
    },

    TriggerScheduled {
        message: String,
        notify: Vec<NotifySource>,
        scheduled_reminder: ReminderSettings,
    },
    Trigger {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReminderActorDelayQueueValue {
    message: String,
    delay: Duration,
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
        let settings = self.shared_actor_state.settings.load();
        for reminder in &settings.reminders {
            let time = reminder.frequency.next_trigger(reminder.starts_on).await;
            let reminder = reminder.clone();

            myself.send_after(Duration::from_secs_f64(time.as_seconds_f64()), move || {
                ReminderActorMessage::TriggerScheduled {
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
            ReminderActorMessage::Set { message, delay } => {
                let v = ReminderActorDelayQueueValue { message, delay };

                self.delay_queue.push(v, delay).await?;
            }
            ReminderActorMessage::Trigger { message } => {
                let message = format!("Reminder about \"{}\"", message);
                notify(&[NotifySource::AndroidApp], message);
            }
            ReminderActorMessage::TriggerScheduled {
                message,
                notify: notify_sources,
                scheduled_reminder,
            } => {
                tracing::info!(
                    "run {} for {scheduled_reminder:?}",
                    scheduled_reminder.state
                );

                if scheduled_reminder.state == ReminderState::Active {
                    notify(&notify_sources, message);
                }

                let time = scheduled_reminder
                    .frequency
                    .next_trigger(scheduled_reminder.starts_on)
                    .await;

                let reminder = scheduled_reminder.clone();
                myself.send_after(Duration::from_secs_f64(time.as_seconds_f64()), move || {
                    ReminderActorMessage::TriggerScheduled {
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
