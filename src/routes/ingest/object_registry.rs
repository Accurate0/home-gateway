use crate::{
    actors::{
        eink_display::{EInkDisplayActor, EInkDisplayMessage},
        synergy::{SynergyActor, SynergyMessage},
    },
    types::{ApiState, AppError},
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use object_registry::types::ObjectEvent;

pub async fn object_registry(
    State(ApiState {
        object_registry, ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(object_registry_payload): Json<ObjectEvent>,
) -> Result<StatusCode, AppError> {
    let bearer_token = headers
        .get(AUTHORIZATION)
        .and_then(|t| t.to_str().ok())
        .map(|t| t.replace("Bearer ", ""))
        .unwrap_or("".to_owned());

    let is_valid_request = object_registry
        .validate_event_token(object_registry::ApiClient::get_jwks, &bearer_token)
        .await?;

    if !is_valid_request {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if object_registry_payload.metadata.namespace != "home-gateway" {
        return Ok(StatusCode::METHOD_NOT_ALLOWED);
    };

    match object_registry_payload.key.as_str() {
        "index.html" => {
            let maybe_actor = ractor::registry::where_is(EInkDisplayActor::NAME.to_string());
            if let Some(actor) = maybe_actor {
                actor.send_message(EInkDisplayMessage::TakeScreenshot)?;
                Ok(StatusCode::CREATED)
            } else {
                Ok(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        "synergy.csv" => {
            let maybe_actor = ractor::registry::where_is(SynergyActor::NAME.to_string());
            if let Some(actor) = maybe_actor {
                let object = object_registry
                    .get_object::<Vec<u8>>("home-gateway", "synergy.csv")
                    .await?;

                actor.send_message(SynergyMessage::NewUpload(object.payload.into()))?;

                Ok(StatusCode::CREATED)
            } else {
                Ok(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        _ => Ok(StatusCode::NOT_FOUND),
    }
}
