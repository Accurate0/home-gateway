use crate::{
    actors::devices::light::{LightHandler, LightHandlerMessage},
    actors::system::rpc,
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    settings::IEEEAddress,
    state::ApiState,
};
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
    pub change: HashMap<IEEEAddress, LightControlChange>,
}

pub async fn light_control(
    State(ApiState { .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(control): Json<LightControlPayload>,
) -> Result<StatusCode, AppError> {
    auth.require(&Scope::new(Resource::Control, Action::Write))
        .map_err(AppError::StatusCode)?;

    for (ieee_addr, change) in control.change {
        let message = match change {
            LightControlChange::Off => LightHandlerMessage::TurnOff { ieee_addr },
            LightControlChange::On => LightHandlerMessage::TurnOn { ieee_addr },
            LightControlChange::Toggle => LightHandlerMessage::Toggle { ieee_addr },
        };

        if let Err(e) = rpc::cast_factory(LightHandler::NAME, message) {
            tracing::error!("error dispatching light control: {e}");
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
