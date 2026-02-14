use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
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
        .map(|t| t.to_str().ok())
        .flatten()
        .map(|t| t.replace("Bearer ", ""))
        .unwrap_or("".to_owned());

    let is_valid_request = object_registry
        .validate_event_token(object_registry::ApiClient::get_jwks, &bearer_token)
        .await?;

    if !is_valid_request {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if object_registry_payload.key == "index.html"
        && object_registry_payload.metadata.namespace == "home-gateway"
    {
        let maybe_actor = ractor::registry::where_is(EInkDisplayActor::NAME.to_string());
        if let Some(actor) = maybe_actor {
            actor.send_message(EInkDisplayMessage::TakeScreenshot)?;
            Ok(StatusCode::CREATED)
        } else {
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
