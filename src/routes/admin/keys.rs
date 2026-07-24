use crate::api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload};
use axum::{
    Json,
    extract::{Path, State},
};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    auth::{Auth, scope::required},
    types::{ApiState, AppError},
};

pub async fn create_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(payload): Json<CreateKeyPayload>,
) -> Result<Json<CreatedKey>, AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE)
        .map_err(AppError::StatusCode)?;

    let created = manager
        .create(&payload.name, &payload.scopes, payload.expires_at)
        .await?;

    Ok(Json(created))
}

pub async fn list_keys(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
) -> Result<impl axum::response::IntoResponse, AppError> {
    auth.require(&required::ADMIN_KEYS_READ)
        .map_err(AppError::StatusCode)?;

    let keys = manager.list().await?;

    Ok(Json(keys))
}

pub async fn update_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKeyPayload>,
) -> Result<Json<ApiKeyInfo>, AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE)
        .map_err(AppError::StatusCode)?;

    let updated = manager
        .update(
            id,
            payload.name.as_deref(),
            payload.scopes.as_deref(),
            payload.expires_at,
        )
        .await?;

    match updated {
        Some(info) => Ok(Json(info)),
        None => Err(AppError::StatusCode(StatusCode::NOT_FOUND)),
    }
}

pub async fn regenerate_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CreatedKey>), AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE)
        .map_err(AppError::StatusCode)?;

    match manager.regenerate(id).await? {
        Some(created) => Ok((StatusCode::CREATED, Json(created))),
        None => Err(AppError::StatusCode(StatusCode::NOT_FOUND)),
    }
}

pub async fn revoke_key(
    State(ApiState { auth: manager, .. }): State<ApiState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require(&required::ADMIN_KEYS_WRITE)
        .map_err(AppError::StatusCode)?;

    if manager.revoke(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
