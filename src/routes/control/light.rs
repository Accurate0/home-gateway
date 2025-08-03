use crate::{
    actors::light::{LightHandler, LightHandlerMessage},
    settings::IEEEAddress,
    types::{ApiState, AppError},
};
use axum::{Json, extract::State};
use http::StatusCode;
use ractor::factory::{FactoryMessage, Job, JobOptions};
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
    Json(control): Json<LightControlPayload>,
) -> Result<StatusCode, AppError> {
    let Some(actor) = ractor::registry::where_is(LightHandler::NAME.to_string()) else {
        tracing::warn!("could not find light actor");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    for (ieee_addr, change) in control.change {
        let message = match change {
            LightControlChange::Off => LightHandlerMessage::TurnOff { ieee_addr },
            LightControlChange::On => LightHandlerMessage::TurnOn { ieee_addr },
            LightControlChange::Toggle => LightHandlerMessage::Toggle { ieee_addr },
        };

        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: message,
            options: JobOptions::default(),
            accepted: None,
        });

        actor.send_message(message)?;
    }

    Ok(StatusCode::NO_CONTENT)
}
