use crate::{
    actors::integrations::synergy::{SynergyActor, SynergyMessage},
    actors::system::rpc,
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
};
use bytes::Bytes;
use http::StatusCode;

pub async fn synergy(Auth(auth): Auth, body: Bytes) -> Result<StatusCode, AppError> {
    auth.require(&Scope::new(Resource::IngestSynergy, Action::Write))
        .map_err(AppError::StatusCode)?;

    match rpc::cast(SynergyActor::NAME, SynergyMessage::NewUpload(body)) {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            tracing::error!("error forwarding synergy event: {e}");
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
