use super::{ReminderActor, ReminderActorDelayQueueValue, ReminderActorMessage};
use crate::delayqueue::{DelayQueue, DelayQueueError};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn reminder_background(
    reminder_delayqueue: DelayQueue<ReminderActorDelayQueueValue>,
    cancellation_token: CancellationToken,
) -> Result<(), DelayQueueError> {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("cancellation requested");
                break Ok(());
            }

            msg = reminder_delayqueue.read(Duration::from_secs(30)) => {
                let maybe_actor = ractor::registry::where_is(ReminderActor::NAME.to_string());
                if let Err(e) = msg {
                    tracing::info!("error in reading from queue: {e}");
                    continue;
                }

                let msg = msg.unwrap();
                if msg.is_none() {
                    continue;
                }

                let msg = msg.unwrap();
                if let Some(actor) = maybe_actor {
                    if let Err(e) = actor.send_message(ReminderActorMessage::TriggerReminder {
                        message: msg.message.message,
                        channel_id: msg.message.channel_id,
                        user_id: vec![msg.message.user_id],
                    }) {
                        tracing::error!("error sending actor message: {e}");
                    }
                }

                reminder_delayqueue.archive(msg.msg_id).await?;
            }
        }
    }
}
