use crate::{
    actors::alarm::{AlarmActor, AlarmMessage, types::AndroidAppAlarmPayload},
    actors::system::rpc,
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
};
use axum::Json;
use http::StatusCode;

pub async fn alarm(Auth(auth): Auth, Json(payload): Json<AndroidAppAlarmPayload>) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::IngestHome, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    if let Err(e) = rpc::cast(AlarmActor::NAME, AlarmMessage::NextAlarm(payload)) {
        tracing::error!("error forwarding alarm event: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}
