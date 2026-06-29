use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{Auth, manager::CreatedKey, scope::required},
    types::{ApiState, AppError},
};

#[derive(Deserialize)]
pub struct CreateKeyPayload {
    pub name: String,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn create_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(payload): Json<CreateKeyPayload>,
) -> Result<Json<CreatedKey>, AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE).map_err(AppError::StatusCode)?;

    let created = manager
        .create(&payload.name, &payload.scopes, payload.expires_at)
        .await?;

    Ok(Json(created))
}

pub async fn list_keys(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
) -> Result<impl axum::response::IntoResponse, AppError> {
    auth.require(&required::ADMIN_KEYS_READ).map_err(AppError::StatusCode)?;

    let keys = manager.list().await?;

    Ok(Json(keys))
}

pub async fn revoke_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE).map_err(AppError::StatusCode)?;

    if manager.revoke(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
