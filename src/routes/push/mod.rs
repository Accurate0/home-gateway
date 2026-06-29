use axum::Json;
use http::StatusCode;
use ractor::factory::{FactoryMessage, Job, JobOptions};
use serde::Deserialize;

use crate::actors::push::{self, PushWorker};
use crate::auth::{Auth, scope::required};

#[derive(Deserialize)]
pub struct PushNotifyPayload {
    #[serde(default = "default_title")]
    pub title: String,
    pub body: String,
}

fn default_title() -> String {
    "Home Gateway".to_string()
}

pub async fn notify(Auth(auth): Auth, Json(payload): Json<PushNotifyPayload>) -> StatusCode {
    if auth.require(&required::REST_PUSH_WRITE).is_err() {
        return StatusCode::FORBIDDEN;
    }

    let Some(actor) = ractor::registry::where_is(PushWorker::NAME.to_string()) else {
        tracing::warn!("push worker not found, cannot send notification");
        return StatusCode::SERVICE_UNAVAILABLE;
    };

    if let Err(e) = actor.send_message(FactoryMessage::Dispatch(Job {
        key: (),
        msg: push::PushMessage::Send {
            title: payload.title,
            body: payload.body,
        },
        options: JobOptions::default(),
        accepted: None,
    })) {
        tracing::error!("error sending to push worker: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}
