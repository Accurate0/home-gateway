use crate::actors::system::rpc;

use tracing::instrument;

use crate::{
    actors::system::push::{self, PushWorker},
    settings::NotifySource,
};

#[instrument]
pub fn notify(notify_sources: &[NotifySource], message: String) {
    for notify in notify_sources {
        match notify {
            NotifySource::AndroidApp => {
                tracing::info!("notifying android app with \"{}\"", message);

                let push_message = push::PushMessage::Send {
                    title: "Home Gateway".to_string(),
                    body: message.clone(),
                };

                if let Err(e) = rpc::cast_factory(PushWorker::NAME, push_message) {
                    tracing::error!("error sending to push worker: {e}");
                };
            }
        }
    }
}
