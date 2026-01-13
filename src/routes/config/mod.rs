#![allow(unused)]
use crate::types::{ApiState, AppError};
use anyhow::Context;
use axum::extract::State;
use bytes::Bytes;
use config_catalog_jwt::verify_jwt;
use http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConfigCatalogRefreshEvent {
    #[allow(unused)]
    key: String,
    payload: serde_yaml::Value,
}

pub async fn refresh(
    State(ApiState { settings, .. }): State<ApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let config_catalog_jwt_secret = std::env::var("CONFIG_CATALOG_JWT_SECRET")?;
    let auth_token = headers
        .get(AUTHORIZATION)
        .map(|h| h.to_str().ok())
        .flatten()
        .context("must have auth header")?
        .replace("Bearer ", "");

    let jwt_verification_result = verify_jwt(config_catalog_jwt_secret.as_bytes(), &auth_token)?;
    tracing::info!("verified jwt for config reload with: {jwt_verification_result:?}");

    let refresh_event = serde_yaml::from_slice::<ConfigCatalogRefreshEvent>(&body)?;

    tracing::info!("new config: {refresh_event:?}");
    // settings.reload(serde_yaml::to_string(&refresh_event.payload)?)?;

    Ok(StatusCode::OK)
}
