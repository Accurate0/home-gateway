use ractor::factory::{FactoryMessage, Job, JobOptions};
use tracing::instrument;

use crate::{
    actors::push::{self, PushWorker},
    settings::NotifySource,
};

#[instrument]
pub fn notify(notify_sources: &[NotifySource], message: String) {
    for notify in notify_sources {
        match notify {
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
