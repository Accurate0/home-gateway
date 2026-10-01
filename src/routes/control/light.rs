use crate::{
    actors::devices::light::{LightHandler, LightHandlerMessage},
    actors::system::rpc,
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
    state::AppState,
};
use anyhow::Context;
use axum::{Json, extract::State};
use http::StatusCode;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LightControlChange {
    Off,
    On,
    Toggle,
}

#[derive(Deserialize)]
pub struct LightControlPayload {
    pub change: HashMap<String, LightControlChange>,
}

pub async fn light_control(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(control): Json<LightControlPayload>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::Light, Action::Write)?;

    let mut messages = Vec::with_capacity(control.change.len());

    for (reference, change) in control.change {
        let ieee_addr = state.devices.address_or_self(&reference).to_owned();

        if state.devices.light(&ieee_addr).is_none() {
            tracing::warn!("rejected light control for `{reference}`, which is not a light");

            return Err(AppError::bad_request(format!(
                "`{reference}` is not a light"
            )));
        }

        messages.push(match change {
            LightControlChange::Off => LightHandlerMessage::TurnOff { ieee_addr },
            LightControlChange::On => LightHandlerMessage::TurnOn { ieee_addr },
            LightControlChange::Toggle => LightHandlerMessage::Toggle { ieee_addr },
        });
    }

    for message in messages {
        rpc::cast_factory(LightHandler::NAME, message).context("dispatching light control")?;
    }

    Ok(StatusCode::NO_CONTENT)
}
