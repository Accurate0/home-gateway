use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

pub const PASSPORT_HEADER: &str = "x-graph-passport";
const PASSPORT_ISSUER: &str = "graph-router";

#[derive(Debug, Clone, Deserialize)]
pub struct PassportConfig {
    pub audience: String,
    pub jwks: String,
}

impl std::str::FromStr for PassportConfig {
    type Err = String;

    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let config: Self =
            serde_json::from_str(json).map_err(|e| format!("SUBGRAPH_AUTH is not valid: {e}"))?;
        if config.audience.is_empty() || config.jwks.is_empty() {
            return Err("SUBGRAPH_AUTH needs an audience and the router's jwks".to_owned());
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Passport {
    identity: Identity,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Identity {
    pub kind: String,
    pub subject: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Clone)]
struct VerifyingKey {
    key: DecodingKey,
    alg: Option<Algorithm>,
}

#[derive(Clone)]
pub struct PassportVerifier {
    inner: Arc<Inner>,
}

struct Inner {
    client: reqwest::Client,
    config: PassportConfig,
    keys: RwLock<BTreeMap<String, VerifyingKey>>,
    last_refresh: Mutex<Option<Instant>>,
}

impl PassportVerifier {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(3600);
    const UNKNOWN_KID_COOLDOWN: Duration = Duration::from_secs(30);

    pub fn new(config: PassportConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(20))
                    .user_agent("home-gateway")
                    .build()
                    .unwrap_or_default(),
                config,
                keys: RwLock::new(BTreeMap::new()),
                last_refresh: Mutex::new(None),
            }),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<Identity, StatusCode> {
        let header = decode_header(token).map_err(|e| {
            tracing::warn!("unreadable passport header: {e}");
            StatusCode::UNAUTHORIZED
        })?;
        let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

        let mut key = self.inner.keys.read().await.get(&kid).cloned();
        if key.is_none() {
            self.refresh_for(&kid).await.map_err(|e| {
                tracing::error!("passport jwks refresh failed: {e}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
            key = self.inner.keys.read().await.get(&kid).cloned();
        }
        let key = key.ok_or(StatusCode::UNAUTHORIZED)?;

        let mut validation = Validation::new(key.alg.unwrap_or(header.alg));
        validation.set_issuer(&[PASSPORT_ISSUER]);
        validation.set_audience(&[&self.inner.config.audience]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.leeway = 5;

        let passport = decode::<Passport>(token, &key.key, &validation)
            .map_err(|e| {
                tracing::warn!("passport rejected: {e}");
                StatusCode::UNAUTHORIZED
            })?
            .claims;

        if passport.identity.kind != "user" {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(passport.identity)
    }

    pub async fn refresh(&self) -> Result<usize, String> {
        let mut last = self.inner.last_refresh.lock().await;
        *last = Some(Instant::now());
        self.fetch_keys().await
    }

    // an in-flight refresh may bring the kid; otherwise refresh at most once per cooldown
    async fn refresh_for(&self, kid: &str) -> Result<(), String> {
        let mut last = self.inner.last_refresh.lock().await;
        let known = self.inner.keys.read().await.contains_key(kid);
        if known || last.is_some_and(|at| at.elapsed() < Self::UNKNOWN_KID_COOLDOWN) {
            return Ok(());
        }

        tracing::info!("unknown passport kid {kid}, refreshing jwks");
        *last = Some(Instant::now());
        self.fetch_keys().await.map(|_| ())
    }

    async fn fetch_keys(&self) -> Result<usize, String> {
        let jwks = &self.inner.config.jwks;

        let set: JwkSet = self
            .inner
            .client
            .get(jwks)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let mut keys = BTreeMap::new();
        for jwk in &set.keys {
            let Some(kid) = jwk.common.key_id.clone() else {
                continue;
            };
            let asymmetric = matches!(
                jwk.algorithm,
                AlgorithmParameters::RSA(_)
                    | AlgorithmParameters::EllipticCurve(_)
                    | AlgorithmParameters::OctetKeyPair(_)
            );
            if !asymmetric {
                continue;
            }
            if let Ok(key) = DecodingKey::from_jwk(jwk) {
                let alg = jwk
                    .common
                    .key_algorithm
                    .and_then(|a| a.to_string().parse::<Algorithm>().ok());
                keys.insert(kid, VerifyingKey { key, alg });
            }
        }

        if keys.is_empty() {
            return Err(format!("no usable keys at {jwks}"));
        }

        let count = keys.len();
        *self.inner.keys.write().await = keys;
        tracing::info!(jwks, keys = count, "refreshed passport jwks");
        Ok(count)
    }

    pub async fn run(self) {
        if let Err(e) = self.refresh().await {
            tracing::warn!("initial passport jwks fetch failed, retrying on demand: {e}");
        }
        let mut ticker = tokio::time::interval(Self::REFRESH_INTERVAL);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = self.refresh().await {
                tracing::warn!("passport jwks refresh failed, keeping previous keys: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    // a throwaway P-256 key, only for signing test passports
    const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgQrxIw5f6gChTjB4J
goqO/XL/jTOI1DURt4md/xUx3qChRANCAARRtn8X4BTpCgfXkHeNNosv447OcYmC
WTbO2iQACXuyX0kacNHv2hA0Kb33waLg/85mkeG1cQVfOEMQq8jA8xNE
-----END PRIVATE KEY-----";
    const TEST_X: &str = "UbZ_F-AU6QoH15B3jTaLL-OOznGJglk2ztokAAl7sl8";
    const TEST_Y: &str = "SRpw0e_aEDQpvffBouD_zmaR4bVxBV84QxCryMDzE0Q";

    pub(crate) async fn verifier() -> PassportVerifier {
        let verifier = PassportVerifier::new(PassportConfig {
            audience: "home".to_owned(),
            jwks: "http://router.invalid/.well-known/passport.jwks".to_owned(),
        });
        let set: JwkSet = serde_json::from_value(serde_json::json!({ "keys": [{
            "kty": "EC", "crv": "P-256", "x": TEST_X, "y": TEST_Y, "kid": "router", "alg": "ES256",
        }] }))
        .unwrap();
        verifier.inner.keys.write().await.insert(
            "router".to_owned(),
            VerifyingKey {
                key: DecodingKey::from_jwk(&set.keys[0]).unwrap(),
                alg: Some(Algorithm::ES256),
            },
        );
        *verifier.inner.last_refresh.lock().await = Some(Instant::now());
        verifier
    }

    pub(crate) fn passport(overrides: serde_json::Value) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut claims = serde_json::json!({
            "iss": PASSPORT_ISSUER,
            "aud": "home",
            "sub": "uuid",
            "iat": now,
            "exp": now + 60,
            "identity": {
                "issuer": "kanidm",
                "kind": "user",
                "subject": "uuid",
                "username": "anurag",
                "groups": ["home_gateway_readers@idm.anurag.sh"],
            },
        });
        for (key, value) in overrides.as_object().unwrap() {
            claims[key] = value.clone();
        }

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some("router".to_owned());
        let key = EncodingKey::from_ec_pem(TEST_KEY.as_bytes()).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    #[tokio::test]
    async fn passports_carry_the_caller_and_their_groups() {
        let identity = verifier()
            .await
            .verify(&passport(serde_json::json!({})))
            .await
            .unwrap();
        assert_eq!(identity.username.as_deref(), Some("anurag"));
        assert_eq!(identity.groups, vec!["home_gateway_readers@idm.anurag.sh"]);
    }

    #[tokio::test]
    async fn passports_for_others_or_from_elsewhere_are_rejected() {
        let verifier = verifier().await;
        for overrides in [
            serde_json::json!({ "aud": "registry" }),
            serde_json::json!({ "iss": "https://idm.anurag.sh/oauth2/openid/graph" }),
            serde_json::json!({ "exp": 1 }),
        ] {
            assert_eq!(
                verifier.verify(&passport(overrides)).await.err(),
                Some(StatusCode::UNAUTHORIZED)
            );
        }
        let forgejo = passport(serde_json::json!({ "identity": {
            "issuer": "forgejo", "kind": "external", "subject": "repo:anurag/k8s",
        } }));
        assert_eq!(
            verifier.verify(&forgejo).await.err(),
            Some(StatusCode::FORBIDDEN)
        );
    }

    #[test]
    fn subgraph_auth_needs_an_audience_and_jwks() {
        let config: PassportConfig =
            r#"{"audience":"home","jwks":"http://router/.well-known/passport.jwks","grants":{}}"#
                .parse()
                .unwrap();
        assert_eq!(config.audience, "home");
        assert!(r#"{"providers":[]}"#.parse::<PassportConfig>().is_err());
    }
}
