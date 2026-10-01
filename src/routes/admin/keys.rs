use axum::{
    Json,
    extract::{Path, State},
};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    auth::{
        AuthContext, AuthManager,
        api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload},
        scope::{Action, Resource, ScopePattern},
    },
    error::AppError,
    repo::api_key::ApiKeyChanges,
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
    auth: AuthContext,
    Json(payload): Json<CreateKeyPayload>,
) -> Result<Json<CreatedKey>, AppError> {
    auth.require(Resource::AdminKeys, Action::Write)?;

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
    auth: AuthContext,
) -> Result<impl axum::response::IntoResponse, AppError> {
    auth.require(Resource::AdminKeys, Action::Read)?;

    let keys = state.handles.expect::<AuthManager>().list().await?;

    Ok(Json(keys))
}

pub async fn update_key(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKeyPayload>,
) -> Result<Json<ApiKeyInfo>, AppError> {
    auth.require(Resource::AdminKeys, Action::Write)?;

    if let Some(scopes) = &payload.scopes {
        validate_scopes(scopes)?;
    }

    let updated = state
        .handles
        .expect::<AuthManager>()
        .update(
            id,
            ApiKeyChanges {
                name: payload.name.as_deref(),
                scopes: payload.scopes.as_deref(),
                expires_at: payload.expires_at,
            },
        )
        .await?;

    match updated {
        Some(info) => Ok(Json(info)),
        None => Err(AppError::StatusCode(StatusCode::NOT_FOUND)),
    }
}

pub async fn regenerate_key(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CreatedKey>), AppError> {
    auth.require(Resource::AdminKeys, Action::Write)?;

    match state.handles.expect::<AuthManager>().regenerate(id).await? {
        Some(created) => Ok((StatusCode::CREATED, Json(created))),
        None => Err(AppError::StatusCode(StatusCode::NOT_FOUND)),
    }
}

pub async fn revoke_key(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::AdminKeys, Action::Write)?;

    if state.handles.expect::<AuthManager>().revoke(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::StatusCode(StatusCode::NOT_FOUND))
    }
}
