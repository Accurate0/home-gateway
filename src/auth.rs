use crate::types::ApiState;
use axum::extract::FromRequestParts;
use http::{StatusCode, request::Parts};

pub struct RequireApiKey;

impl FromRequestParts<ApiState> for RequireApiKey {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        if cfg!(debug_assertions) {
            return Ok(Self);
        }

        let api_key = parts
            .headers
            .get("X-Api-Key")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.trim());

        match api_key {
            Some(api_key) if api_key == state.settings.load().api_key => Ok(Self),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
