use crate::{
    actors::integrations::synergy::{SynergyActor, SynergyMessage},
    actors::system::rpc,
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
};
use anyhow::Context;
use bytes::Bytes;
use http::StatusCode;

pub async fn synergy(auth: AuthContext, body: Bytes) -> Result<StatusCode, AppError> {
    auth.require(Resource::IngestSynergy, Action::Write)?;

    rpc::cast(SynergyActor::NAME, SynergyMessage::NewUpload(body))
        .context("forwarding synergy upload")?;

    Ok(StatusCode::ACCEPTED)
}
