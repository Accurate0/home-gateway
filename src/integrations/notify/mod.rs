use crate::actors::system::rpc;

use tracing::instrument;

use crate::{
    actors::system::push::{self, PushNotification, PushWorker, types::PushAction},
    settings::{NotifyCategory, NotifySource},
};

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub category: NotifyCategory,
    pub tag: String,
    pub actions: Vec<PushAction>,
}

impl Notification {
    pub fn new(body: String, category: NotifyCategory, tag: String) -> Self {
        Self {
            title: "Home Gateway".to_string(),
            body,
            category,
            tag,
            actions: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_actions(mut self, actions: Vec<PushAction>) -> Self {
        self.actions = actions;
        self
    }
}

#[instrument]
pub fn notify(notify_sources: &[NotifySource], notification: Notification) {
    for notify in notify_sources {
        match notify {
            NotifySource::AndroidApp => {
                tracing::info!("notifying android app with \"{}\"", notification.body);

                let push_message = push::PushMessage::Send(PushNotification {
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                    category: notification.category,
                    tag: notification.tag.clone(),
                    actions: notification.actions.clone(),
                });

                if let Err(e) = rpc::cast_factory(PushWorker::NAME, push_message) {
                    tracing::error!("error sending to push worker: {e}");
                };
            }
        }
    }
}
