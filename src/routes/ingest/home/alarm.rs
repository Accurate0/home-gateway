use crate::{
    actors::alarm::{AlarmActor, AlarmMessage, types::AndroidAppAlarmPayload},
    actors::system::rpc,
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
};
use anyhow::Context;
use axum::Json;
use http::StatusCode;

pub async fn alarm(
    auth: AuthContext,
    Json(payload): Json<AndroidAppAlarmPayload>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::IngestHome, Action::Write)?;

    rpc::cast(AlarmActor::NAME, AlarmMessage::NextAlarm(payload))
        .context("forwarding alarm event")?;

    Ok(StatusCode::NO_CONTENT)
}
