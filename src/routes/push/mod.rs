use axum::Json;
use http::StatusCode;

use serde::Deserialize;

use crate::actors::system::push::{self, PushWorker};
use crate::actors::system::rpc;
use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};

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
    if auth
        .require(&Scope::new(Resource::Push, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let message = push::PushMessage::Send {
        title: payload.title,
        body: payload.body,
    };

    if let Err(e) = rpc::cast_factory(PushWorker::NAME, message) {
        tracing::error!("error sending to push worker: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}
