use std::{collections::HashSet, str::FromStr, sync::Arc, time::Duration};

use http::StatusCode;
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, EllipticCurve, Jwk, JwkSet},
};
use moka::future::Cache;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::{http::get_traced_http_client, settings::OAuthSettings};

use super::AuthContext;

const CACHE_CAPACITY: u64 = 32;
const CACHE_TTL: Duration = Duration::from_secs(3600);

const USERINFO_CACHE_CAPACITY: u64 = 256;
const USERINFO_CACHE_TTL: Duration = Duration::from_secs(15 * 60);

/// Claims we read out of a Kanidm access token. `aud`/`iss`/`exp` are validated
/// by `jsonwebtoken` itself; the access token only carries identity, so group
/// membership is fetched separately from the userinfo endpoint.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

/// Identity and group membership from the OIDC userinfo endpoint. Kanidm omits
/// `groups` from the access token, so it must be read from userinfo. `groups`
/// is deserialized dynamically since the claim name is configurable.
#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

pub struct VerifyingKey {
    key: DecodingKey,
    alg: Algorithm,
}

pub struct OAuthValidator {
    settings: OAuthSettings,
    http: ClientWithMiddleware,
    // kid -> verifying key. Missing/rotated keys trigger a JWKS refetch.
    keys: Cache<String, Arc<VerifyingKey>>,
    // bearer token -> userinfo, to avoid a userinfo round-trip on every request.
    userinfo: Cache<String, Arc<UserInfo>>,
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
            userinfo: Cache::builder()
                .max_capacity(USERINFO_CACHE_CAPACITY)
                .time_to_live(USERINFO_CACHE_TTL)
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
    async fn key_for(&self, kid: &str) -> Result<Arc<VerifyingKey>, StatusCode> {
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
            tracing::error!("invalid jwt header: {e}");
            StatusCode::UNAUTHORIZED
        })?;
        let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;
        let key = self.key_for(&kid).await?;

        if header.alg != key.alg {
            tracing::warn!(
                "jwt for kid {kid} declares alg {:?} but the jwks key signs {:?}",
                header.alg,
                key.alg
            );
            return Err(StatusCode::UNAUTHORIZED);
        }

        let mut validation = Validation::new(key.alg);
        validation.set_issuer(&[&self.settings.issuer]);
        validation.set_audience(&[&self.settings.audience]);

        let claims = decode::<Claims>(token, &key.key, &validation)
            .map_err(|e| {
                tracing::error!("jwt validation failed: {e}");
                StatusCode::UNAUTHORIZED
            })?
            .claims;

        let userinfo = self.fetch_userinfo(token).await?;

        let scopes = self.scopes_for(&userinfo);
        if scopes.is_empty() {
            tracing::error!("no scopes found");
            return Err(StatusCode::FORBIDDEN);
        }

        let name = userinfo.preferred_username.clone().or(Some(claims.sub));
        Ok(AuthContext::from_scopes(None, name, &scopes))
    }

    /// Fetch the caller's userinfo from the OIDC endpoint using their bearer
    /// token. Kanidm does not include `groups` in the access token, so this is
    /// where group membership comes from. Results are cached per token for
    /// `USERINFO_CACHE_TTL` to avoid a round-trip on every request.
    async fn fetch_userinfo(&self, token: &str) -> Result<Arc<UserInfo>, StatusCode> {
        if let Some(userinfo) = self.userinfo.get(token).await {
            return Ok(userinfo);
        }

        let userinfo = Arc::new(self.request_userinfo(token).await?);
        self.userinfo
            .insert(token.to_owned(), userinfo.clone())
            .await;
        Ok(userinfo)
    }

    async fn request_userinfo(&self, token: &str) -> Result<UserInfo, StatusCode> {
        self.http
            .get(&self.settings.userinfo_url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("failed to fetch userinfo: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .error_for_status()
            .map_err(|e| {
                tracing::error!("userinfo request failed: {e}");
                StatusCode::UNAUTHORIZED
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!("failed to parse userinfo: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Map the userinfo group claim through `group_scopes`, flattening and
    /// de-duplicating the granted scope strings.
    fn scopes_for(&self, userinfo: &UserInfo) -> Vec<String> {
        let groups = userinfo
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
fn decoding_key(jwk: &Jwk) -> Result<VerifyingKey, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_jwk(jwk)?;
    let alg = jwk_algorithm(jwk)?;

    Ok(VerifyingKey { key, alg })
}

fn jwk_algorithm(jwk: &Jwk) -> Result<Algorithm, jsonwebtoken::errors::Error> {
    if let Some(declared) = jwk.common.key_algorithm {
        return Algorithm::from_str(&declared.to_string());
    }

    match &jwk.algorithm {
        AlgorithmParameters::RSA(_) => Ok(Algorithm::RS256),
        AlgorithmParameters::EllipticCurve(ec) => match ec.curve {
            EllipticCurve::P256 => Ok(Algorithm::ES256),
            EllipticCurve::P384 => Ok(Algorithm::ES384),
            _ => Err(jsonwebtoken::errors::ErrorKind::InvalidAlgorithm.into()),
        },
        AlgorithmParameters::OctetKeyPair(_) => Ok(Algorithm::EdDSA),
        _ => Err(jsonwebtoken::errors::ErrorKind::InvalidAlgorithm.into()),
    }
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
                userinfo_url: "http://userinfo".into(),
                audience: "home-gateway".into(),
                groups_claim: "groups".into(),
                group_scopes,
            },
            http: get_traced_http_client().unwrap(),
            keys: Cache::builder().build(),
            userinfo: Cache::builder().build(),
        }
    }

    fn userinfo(groups: &[&str]) -> UserInfo {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "groups".into(),
            serde_json::json!(groups.iter().collect::<Vec<_>>()),
        );
        UserInfo {
            preferred_username: Some("user".into()),
            extra,
        }
    }

    #[test]
    fn known_group_maps_to_scopes() {
        let v = validator(HashMap::from([(
            "admins@idm".to_owned(),
            vec!["**:*".to_owned()],
        )]));
        assert_eq!(v.scopes_for(&userinfo(&["admins@idm"])), vec!["**:*"]);
    }

    #[test]
    fn multiple_groups_dedupe() {
        let v = validator(HashMap::from([
            ("a".to_owned(), vec!["light:read".to_owned()]),
            (
                "b".to_owned(),
                vec!["light:read".to_owned(), "epd:read".to_owned()],
            ),
        ]));
        let mut scopes = v.scopes_for(&userinfo(&["a", "b"]));
        scopes.sort();
        assert_eq!(scopes, vec!["epd:read", "light:read"]);
    }

    #[test]
    fn unknown_group_yields_no_scopes() {
        let v = validator(HashMap::from([(
            "admins@idm".to_owned(),
            vec!["**:*".to_owned()],
        )]));
        assert!(v.scopes_for(&userinfo(&["nobody@idm"])).is_empty());
    }

    fn jwk(value: serde_json::Value) -> Jwk {
        serde_json::from_value(value).expect("jwk")
    }

    #[test]
    fn declared_jwk_algorithm_wins() {
        let jwk = jwk(serde_json::json!({
            "kty": "RSA",
            "alg": "PS512",
            "n": "sXchYQ",
            "e": "AQAB",
        }));

        assert_eq!(jwk_algorithm(&jwk).unwrap(), Algorithm::PS512);
    }

    #[test]
    fn rsa_jwk_without_alg_defaults_to_rs256() {
        let jwk = jwk(serde_json::json!({
            "kty": "RSA",
            "n": "sXchYQ",
            "e": "AQAB",
        }));

        assert_eq!(jwk_algorithm(&jwk).unwrap(), Algorithm::RS256);
    }

    #[test]
    fn ec_jwk_algorithm_follows_the_curve() {
        let jwk = jwk(serde_json::json!({
            "kty": "EC",
            "crv": "P-384",
            "x": "sXchYQ",
            "y": "sXchYQ",
        }));

        assert_eq!(jwk_algorithm(&jwk).unwrap(), Algorithm::ES384);
    }

    #[test]
    fn symmetric_jwk_is_rejected() {
        let jwk = jwk(serde_json::json!({
            "kty": "oct",
            "k": "sXchYQ",
        }));

        assert!(jwk_algorithm(&jwk).is_err());
    }
}
