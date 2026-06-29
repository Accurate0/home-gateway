use crate::{
    actors::alarm::{AlarmActor, AlarmMessage, types::AndroidAppAlarmPayload},
    auth::{Auth, scope::required},
};
use axum::Json;
use http::StatusCode;

pub async fn alarm(Auth(auth): Auth, Json(payload): Json<AndroidAppAlarmPayload>) -> StatusCode {
    if auth.require(&required::INGEST_HOME_WRITE).is_err() {
        return StatusCode::FORBIDDEN;
    }

    let Some(actor) = ractor::registry::where_is(AlarmActor::NAME.to_string()) else {
        tracing::warn!("alarm actor not found");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    if let Err(e) = actor.send_message(AlarmMessage::NextAlarm(payload)) {
        tracing::error!("error forwarding alarm event {e}")
    };

    StatusCode::NO_CONTENT
}
