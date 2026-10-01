pub mod api_types;
pub mod client_ip;
pub mod context;
pub mod credentials;
pub mod expiry_change;
pub mod generated_key;
pub mod lockout;
pub mod manager;
pub mod missing_scope;
pub mod oauth;
pub mod scope;

pub use client_ip::{ClientIp, client_ip};
pub use context::AuthContext;
pub use credentials::Credentials;
pub use expiry_change::ExpiryChange;
pub use generated_key::GeneratedKey;
pub use lockout::AuthLockout;
pub use manager::AuthManager;
pub use missing_scope::MissingScope;
pub use oauth::OAuthValidator;

use std::net::IpAddr;

use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use http::{StatusCode, request::Parts};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::state::AppState;

fn dev_bypass_enabled() -> bool {
    std::env::var("AUTH_DEV_BYPASS").is_ok_and(|value| value == "1")
}

pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

async fn resolve_api_key(
    api_key: &str,
    state: &AppState,
) -> Result<Option<AuthContext>, StatusCode> {
    let legacy_key = state.settings.api_key.as_bytes();

    if !legacy_key.is_empty() && bool::from(api_key.as_bytes().ct_eq(legacy_key)) {
        return Ok(Some(AuthContext::full_access()));
    }

    let manager = state.handles.expect::<AuthManager>();
    let hashed = hash_key(api_key);

    let key = manager.lookup_by_hash(&hashed).await.map_err(|e| {
        tracing::error!("failed to look up api key: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(key) = key {
        if key.revoked_at.is_some() {
            return Err(StatusCode::UNAUTHORIZED);
        }
        if let Some(expires_at) = key.expires_at
            && expires_at <= Utc::now()
        {
            return Err(StatusCode::UNAUTHORIZED);
        }

        manager.touch_last_used(key.id).await;

        return Ok(Some(AuthContext::from_scopes(
            Some(key.id),
            Some(key.name.clone()),
            &key.scopes,
        )));
    }

    Ok(None)
}

async fn resolve_credentials(
    credentials: Credentials<'_>,
    state: &AppState,
) -> Result<AuthContext, StatusCode> {
    if dev_bypass_enabled() {
        return Ok(AuthContext::full_access());
    }

    if let Some(api_key) = credentials.api_key
        && let Some(auth) = resolve_api_key(api_key, state).await?
    {
        return Ok(auth);
    }

    if let Some(token) = credentials.bearer
        && let Some(result) = state
            .handles
            .expect::<AuthManager>()
            .validate_oauth(token)
            .await
    {
        return result;
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn resolve_auth(
    credentials: Credentials<'_>,
    ip: Option<IpAddr>,
    state: &AppState,
) -> Result<AuthContext, StatusCode> {
    let result = resolve_credentials(credentials, state).await;

    if result.as_ref().err() == Some(&StatusCode::UNAUTHORIZED)
        && let Some(ip) = ip
    {
        state
            .handles
            .expect::<AuthManager>()
            .lockout()
            .record_failure(ip)
            .await;
    }

    result
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let lockout = state.handles.expect::<AuthManager>().lockout();
    let ip = client_ip(req.headers(), req.extensions());

    if let Some(ip) = ip
        && lockout.is_locked(ip).await
    {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    let credentials = Credentials::from_headers(req.headers());

    match resolve_auth(credentials, ip, &state).await {
        Ok(auth) => {
            req.extensions_mut().insert(auth);
            next.run(req).await
        }
        Err(status) => status.into_response(),
    }
}

impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
