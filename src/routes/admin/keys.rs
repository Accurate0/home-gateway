use crate::auth::AuthManager;
use crate::auth::api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload};
use axum::{
    Json,
    extract::{Path, State},
};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope, ScopePattern},
    },
    error::AppError,
    state::AppState,
};

fn validate_scopes(scopes: &[String]) -> Result<(), AppError> {
    for scope in scopes {
        if let Err(e) = ScopePattern::parse(scope) {
            tracing::warn!("rejecting api key with invalid scope '{scope}': {e}");
            return Err(AppError::StatusCode(StatusCode::BAD_REQUEST));
        }
    }

    Ok(())
}

pub async fn create_key(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Json(payload): Json<CreateKeyPayload>,
) -> Result<Json<CreatedKey>, AppError> {
    auth.require(&Scope::new(Resource::AdminKeys, Action::Write))
        .map_err(AppError::StatusCode)?;

    validate_scopes(&payload.scopes)?;

    let created = state
        .handles
        .expect::<AuthManager>()
        .create(&payload.name, &payload.scopes, payload.expires_at)
        .await?;

    Ok(Json(created))
}

pub async fn list_keys(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<impl axum::response::IntoResponse, AppError> {
    auth.require(&Scope::new(Resource::AdminKeys, Action::Read))
        .map_err(AppError::StatusCode)?;

    let keys = state.handles.expect::<AuthManager>().list().await?;

    Ok(Json(keys))
}

pub async fn update_key(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKeyPayload>,
) -> Result<Json<ApiKeyInfo>, AppError> {
    auth.require(&Scope::new(Resource::AdminKeys, Action::Write))
        .map_err(AppError::StatusCode)?;

    if let Some(scopes) = &payload.scopes {
        validate_scopes(scopes)?;
    }

    let updated = state
        .handles
        .expect::<AuthManager>()
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
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CreatedKey>), AppError> {
    auth.require(&Scope::new(Resource::AdminKeys, Action::Write))
        .map_err(AppError::StatusCode)?;

    match state.handles.expect::<AuthManager>().regenerate(id).await? {
        Some(created) => Ok((StatusCode::CREATED, Json(created))),
        None => Err(AppError::StatusCode(StatusCode::NOT_FOUND)),
    }
}

pub async fn revoke_key(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require(&Scope::new(Resource::AdminKeys, Action::Write))
        .map_err(AppError::StatusCode)?;

    if state.handles.expect::<AuthManager>().revoke(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
