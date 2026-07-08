use std::{collections::HashSet, sync::Arc, time::Duration};

use http::StatusCode;
use jsonwebtoken::{
    DecodingKey, Validation, decode, decode_header,
    jwk::{Jwk, JwkSet},
};
use moka::future::Cache;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::{http::get_traced_http_client, settings::OAuthSettings};

use super::AuthContext;

const CACHE_CAPACITY: u64 = 32;
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Claims we read out of a Kanidm access token. `aud`/`iss`/`exp` are validated
/// by `jsonwebtoken` itself; here we only need identity and group membership.
/// `groups` is deserialized dynamically since the claim name is configurable.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

pub struct OAuthValidator {
    settings: OAuthSettings,
    http: ClientWithMiddleware,
    // kid -> decoding key. Missing/rotated keys trigger a JWKS refetch.
    keys: Cache<String, Arc<DecodingKey>>,
}

impl OAuthValidator {
    pub fn new(settings: OAuthSettings) -> Result<Self, crate::http::HttpCreationError> {
        Ok(Self {
            settings,
            http: get_traced_http_client()?,
            keys: Cache::builder()
                .max_capacity(CACHE_CAPACITY)
                .time_to_live(CACHE_TTL)
                .build(),
        })
    }

    async fn refresh_jwks(&self) -> Result<(), StatusCode> {
        let set: JwkSet = self
            .http
            .get(&self.settings.jwks_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("failed to fetch jwks: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!("failed to parse jwks: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for jwk in &set.keys {
            if let Some(kid) = jwk.common.key_id.clone() {
                match decoding_key(jwk) {
                    Ok(key) => self.keys.insert(kid, Arc::new(key)).await,
                    Err(e) => tracing::warn!("ignoring unusable jwk {kid}: {e}"),
                }
            }
        }

        Ok(())
    }

    /// Returns the decoding key for `kid`, refetching the JWKS once on a miss so
    /// key rotation is picked up without a restart.
    async fn key_for(&self, kid: &str) -> Result<Arc<DecodingKey>, StatusCode> {
        if let Some(key) = self.keys.get(kid).await {
            return Ok(key);
        }

        self.refresh_jwks().await?;

        self.keys.get(kid).await.ok_or_else(|| {
            tracing::warn!("no jwks key for kid {kid}");
            StatusCode::UNAUTHORIZED
        })
    }

    /// Validate a bearer JWT and turn the caller's groups into an `AuthContext`.
    /// Returns 401 for an invalid token and 403 when no group maps to any scope.
    pub async fn validate(&self, token: &str) -> Result<AuthContext, StatusCode> {
        let header = decode_header(token).map_err(|e| {
            tracing::debug!("invalid jwt header: {e}");
            StatusCode::UNAUTHORIZED
        })?;
        let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;
        let key = self.key_for(&kid).await?;

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.settings.issuer]);
        validation.set_audience(&[&self.settings.audience]);

        let claims = decode::<Claims>(token, &key, &validation)
            .map_err(|e| {
                tracing::debug!("jwt validation failed: {e}");
                StatusCode::UNAUTHORIZED
            })?
            .claims;

        let scopes = self.scopes_for(&claims);
        if scopes.is_empty() {
            return Err(StatusCode::FORBIDDEN);
        }

        let name = claims.preferred_username.clone().or(Some(claims.sub));
        Ok(AuthContext::from_scopes(None, name, &scopes))
    }

    /// Map the token's group claim through `group_scopes`, flattening and
    /// de-duplicating the granted scope strings.
    fn scopes_for(&self, claims: &Claims) -> Vec<String> {
        let groups = claims
            .extra
            .get(&self.settings.groups_claim)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let mut seen = HashSet::new();
        groups
            .iter()
            .filter_map(|g| self.settings.group_scopes.get(*g))
            .flatten()
            .filter(|s| seen.insert((*s).clone()))
            .cloned()
            .collect()
    }
}

/// Kanidm uses RSA (RS256) signing keys by default but may rotate to EC; let
/// `jsonwebtoken` derive the key from whatever the JWK declares.
fn decoding_key(jwk: &Jwk) -> Result<DecodingKey, jsonwebtoken::errors::Error> {
    DecodingKey::from_jwk(jwk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn validator(group_scopes: HashMap<String, Vec<String>>) -> OAuthValidator {
        OAuthValidator {
            settings: OAuthSettings {
                issuer: "iss".into(),
                jwks_url: "http://jwks".into(),
                audience: "home-gateway".into(),
                groups_claim: "groups".into(),
                group_scopes,
            },
            http: get_traced_http_client().unwrap(),
            keys: Cache::builder().build(),
        }
    }

    fn claims(groups: &[&str]) -> Claims {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "groups".into(),
            serde_json::json!(groups.iter().collect::<Vec<_>>()),
        );
        Claims {
            sub: "user".into(),
            preferred_username: Some("user".into()),
            extra,
        }
    }

    #[test]
    fn known_group_maps_to_scopes() {
        let v = validator(HashMap::from([(
            "admins@idm".to_owned(),
            vec!["*".to_owned()],
        )]));
        assert_eq!(v.scopes_for(&claims(&["admins@idm"])), vec!["*"]);
    }

    #[test]
    fn multiple_groups_dedupe() {
        let v = validator(HashMap::from([
            ("a".to_owned(), vec!["graphql:*:read".to_owned()]),
            (
                "b".to_owned(),
                vec!["graphql:*:read".to_owned(), "rest:epd:read".to_owned()],
            ),
        ]));
        let mut scopes = v.scopes_for(&claims(&["a", "b"]));
        scopes.sort();
        assert_eq!(scopes, vec!["graphql:*:read", "rest:epd:read"]);
    }

    #[test]
    fn unknown_group_yields_no_scopes() {
        let v = validator(HashMap::from([(
            "admins@idm".to_owned(),
            vec!["*".to_owned()],
        )]));
        assert!(v.scopes_for(&claims(&["nobody@idm"])).is_empty());
    }
}
