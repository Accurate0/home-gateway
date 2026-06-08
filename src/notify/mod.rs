use itertools::Itertools;
use ractor::factory::{FactoryMessage, Job, JobOptions};
use tracing::instrument;

use crate::{
    actors::{
        push::{self, PushWorker},
        selfbot::{self, SelfBotWorker},
    },
    settings::NotifySource,
};

#[instrument]
pub fn notify(notify_sources: &[NotifySource], message: String, formatting: bool) {
    for notify in notify_sources {
        match notify {
            NotifySource::Discord {
                channel_id,
                mentions,
            } => {
                let Some(actor) = ractor::registry::where_is(SelfBotWorker::NAME.to_string())
                else {
                    tracing::warn!("selfbot worker not found, skipping discord notification");
                    continue;
                };

                tracing::info!("notifying: {channel_id} with \"{}\"", message);
                let mentions = mentions.iter().map(|id| format!("<@{id}>")).join(" ");

                let message_with_mentions = if formatting {
                    format!("> {mentions} **{message}**")
                } else {
                    format!("{mentions} {message}")
                };

                if let Err(e) = actor.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: selfbot::SelfBotMessage::SendMessage(
                        selfbot::types::SelfBotMessageRequest {
                            message: message_with_mentions,
                            channel_id: if cfg!(debug_assertions) {
                                1392070912609751131 // testing gc
                            } else {
                                *channel_id
                            },
                        },
                    ),
                    options: JobOptions::default(),
                    accepted: None,
                })) {
                    tracing::error!("error sending to selfbot: {e}");
                };
            }
            NotifySource::AndroidApp => {
                let Some(actor) = ractor::registry::where_is(PushWorker::NAME.to_string()) else {
                    tracing::warn!("push worker not found, skipping android app notification");
                    continue;
                };

                tracing::info!("notifying android app with \"{}\"", message);

                if let Err(e) = actor.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: push::PushMessage::Send {
                        title: "Home Gateway".to_string(),
                        body: message.clone(),
                    },
                    options: JobOptions::default(),
                    accepted: None,
                })) {
                    tracing::error!("error sending to push worker: {e}");
                };
            }
        }
    }
}
