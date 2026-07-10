pub mod context;
pub mod manager;
pub mod oauth;
pub mod scope;

pub use context::AuthContext;
pub use manager::AuthManager;
pub use oauth::OAuthValidator;

use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use http::{HeaderMap, StatusCode, request::Parts};
use sha2::{Digest, Sha256};

use crate::types::ApiState;

pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

async fn resolve_api_key(
    api_key: &str,
    state: &ApiState,
) -> Result<Option<AuthContext>, StatusCode> {
    let settings = &state.settings;

    if !settings.api_key.is_empty() && api_key == settings.api_key {
        return Ok(Some(AuthContext::full_access(true)));
    }

    let hashed = hash_key(api_key);
    let key = state.auth.lookup_by_hash(&hashed).await.map_err(|e| {
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

        state.auth.touch_last_used(key.id);

        return Ok(Some(AuthContext::from_scopes(
            Some(key.id),
            Some(key.name.clone()),
            &key.scopes,
        )));
    }

    Ok(None)
}

pub async fn resolve_ws_auth(
    token: Option<&str>,
    state: &ApiState,
) -> Result<AuthContext, StatusCode> {
    if cfg!(debug_assertions) {
        return Ok(AuthContext::full_access(false));
    }

    let token = token.map(|s| s.trim()).filter(|s| !s.is_empty());

    if let Some(token) = token {
        if let Some(auth) = resolve_api_key(token, state).await? {
            return Ok(auth);
        }
        // ws sends a single token field; a JWT (has dots) is an OAuth access token.
        if token.contains('.')
            && let Some(oauth) = &state.auth.oauth
        {
            return oauth.validate(token).await;
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn resolve_auth(
    headers: &HeaderMap,
    state: &ApiState,
) -> Result<AuthContext, StatusCode> {
    if cfg!(debug_assertions) {
        return Ok(AuthContext::full_access(false));
    }

    let settings = &state.settings;

    let api_key = headers
        .get("X-Api-Key")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.trim());

    if let Some(api_key) = api_key
        && let Some(auth) = resolve_api_key(api_key, state).await?
    {
        return Ok(auth);
    }

    let bearer = headers
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if let Some(token) = bearer
        && let Some(oauth) = &state.auth.oauth
    {
        return oauth.validate(token).await;
    }

    let webhook_secret = headers
        .get("X-Webhook-Secret")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.trim());

    if let Some(secret) = webhook_secret {
        if !settings.android_app_webhook_secret.is_empty()
            && secret == settings.android_app_webhook_secret
        {
            return Ok(AuthContext::from_scopes(
                None,
                Some("android-webhook".to_owned()),
                &["ingest:home:write".to_owned()],
            ));
        }

        if !settings.unifi_webhook_secret.is_empty() && secret == settings.unifi_webhook_secret {
            return Ok(AuthContext::from_scopes(
                None,
                Some("unifi-webhook".to_owned()),
                &["ingest:unifi:write".to_owned()],
            ));
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(
    State(state): State<ApiState>,
    mut req: Request,
    next: Next,
) -> Response {
    let query_auth = req.uri().query().and_then(|query| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == "auth")
            .map(|(_, value)| value.to_owned())
    });

    let resolved = if let Some(token) = query_auth
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        match resolve_api_key(token, &state).await {
            Ok(Some(auth)) => Ok(auth),
            Ok(None) => resolve_auth(req.headers(), &state).await,
            Err(status) => Err(status),
        }
    } else {
        resolve_auth(req.headers(), &state).await
    };

    match resolved {
        Ok(auth) => {
            req.extensions_mut().insert(auth);
            next.run(req).await
        }
        Err(status) => status.into_response(),
    }
}

pub struct Auth(pub AuthContext);

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .map(Auth)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
