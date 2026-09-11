use std::{collections::HashSet, str::FromStr, sync::Arc};

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

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

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
    keys: Cache<String, Arc<VerifyingKey>>,
    userinfo: Cache<String, Arc<UserInfo>>,
}

impl OAuthValidator {
    pub fn new(
        settings: OAuthSettings,
        timeout: std::time::Duration,
    ) -> Result<Self, crate::http::HttpCreationError> {
        let keys = Cache::builder()
            .max_capacity(settings.cache.keys.capacity)
            .time_to_live(settings.cache.keys.ttl())
            .build();

        let userinfo = Cache::builder()
            .max_capacity(settings.cache.userinfo.capacity)
            .time_to_live(settings.cache.userinfo.ttl())
            .build();

        Ok(Self {
            settings,
            http: get_traced_http_client(timeout)?,
            keys,
            userinfo,
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
                cache: crate::settings::OAuthCacheSettings {
                    keys: crate::settings::CacheSettings {
                        capacity: 32,
                        ttl: chrono::TimeDelta::hours(1),
                    },
                    userinfo: crate::settings::CacheSettings {
                        capacity: 256,
                        ttl: chrono::TimeDelta::minutes(15),
                    },
                },
            },
            http: get_traced_http_client(std::time::Duration::from_secs(30)).unwrap(),
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
