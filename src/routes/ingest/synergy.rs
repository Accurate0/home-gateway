use crate::{
    actors::synergy::{SynergyActor, SynergyMessage},
    auth::{Auth, scope::required},
    types::AppError,
};
use bytes::Bytes;
use http::StatusCode;

pub async fn synergy(Auth(auth): Auth, body: Bytes) -> Result<StatusCode, AppError> {
    auth.require(&required::INGEST_SYNERGY_WRITE)
        .map_err(AppError::StatusCode)?;

    let Some(actor) = ractor::registry::where_is(SynergyActor::NAME.to_string()) else {
        tracing::warn!("synergy actor not found");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    match actor.send_message(SynergyMessage::NewUpload(body)) {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            tracing::error!("error forwarding synergy event {e}");
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
