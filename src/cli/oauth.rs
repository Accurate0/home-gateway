use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use rand::RngExt;
use rand::distr::Alphanumeric;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;

use super::credentials::Credentials;

pub const DEFAULT_ISSUER: &str = "https://idm.anurag.sh/oauth2/openid/home-gateway";
pub const DEFAULT_CLIENT_ID: &str = "home-gateway";

const SCOPES: &str = "openid email profile groups";
const VERIFIER_LEN: usize = 64;
const STATE_LEN: usize = 32;
const LOGIN_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("failed to reach the identity provider: {0}")]
    Request(#[from] reqwest_middleware::Error),
    #[error("failed to read the identity provider response: {0}")]
    Decode(#[from] reqwest::Error),
    #[error("openid discovery failed at {url}: {status}")]
    Discovery { url: String, status: String },
    #[error("token request rejected: {0}")]
    Token(String),
    #[error("the identity provider returned an error: {0}")]
    Callback(String),
    #[error("callback state did not match, the login may have been tampered with")]
    StateMismatch,
    #[error("timed out waiting for the browser login to complete")]
    Timeout,
    #[error("failed to start the local callback server: {0}")]
    Listener(std::io::Error),
    #[error("no refresh token is stored, run `home login`")]
    NoRefreshToken,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderMetadata {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate_pkce() -> Pkce {
    let verifier: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(VERIFIER_LEN)
        .map(char::from)
        .collect();

    let challenge = challenge_for(&verifier);

    Pkce {
        verifier,
        challenge,
    }
}

pub fn challenge_for(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn random_state() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(STATE_LEN)
        .map(char::from)
        .collect()
}

pub async fn discover(
    http: &ClientWithMiddleware,
    issuer: &str,
) -> Result<ProviderMetadata, OAuthError> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        issuer.trim_end_matches('/')
    );

    let response = http.get(&url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(OAuthError::Discovery {
            url,
            status: status.to_string(),
        });
    }

    Ok(response.json().await?)
}

pub fn authorize_url(
    metadata: &ProviderMetadata,
    client_id: &str,
    redirect_uri: &str,
    pkce: &Pkce,
    state: &str,
) -> String {
    let query = [
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("scope", SCOPES),
        ("state", state),
        ("code_challenge", pkce.challenge.as_str()),
        ("code_challenge_method", "S256"),
    ]
    .iter()
    .map(|(key, value)| format!("{key}={}", urlencode(value)))
    .collect::<Vec<_>>()
    .join("&");

    format!("{}?{query}", metadata.authorization_endpoint)
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[derive(Debug)]
pub struct Callback {
    pub code: String,
    pub state: String,
}

pub fn resolve_callback(
    params: &HashMap<String, String>,
    expected_state: &str,
) -> Result<Callback, OAuthError> {
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(|d| format!("{error}: {d}"))
            .unwrap_or_else(|| error.clone());

        return Err(OAuthError::Callback(description));
    }

    let code = params
        .get("code")
        .ok_or_else(|| OAuthError::Callback("no code in the callback".to_owned()))?;
    let state = params
        .get("state")
        .ok_or_else(|| OAuthError::Callback("no state in the callback".to_owned()))?;

    if state != expected_state {
        return Err(OAuthError::StateMismatch);
    }

    Ok(Callback {
        code: code.clone(),
        state: state.clone(),
    })
}

type CallbackSender = Arc<Mutex<Option<oneshot::Sender<Result<Callback, OAuthError>>>>>;

#[derive(Clone)]
struct CallbackState {
    expected_state: String,
    sender: CallbackSender,
}

async fn callback_handler(
    State(state): State<CallbackState>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<&'static str> {
    let outcome = resolve_callback(&params, &state.expected_state);
    let body = if outcome.is_ok() {
        "<html><body><h1>signed in</h1><p>you can close this tab and return to your terminal.</p></body></html>"
    } else {
        "<html><body><h1>sign-in failed</h1><p>check your terminal for details.</p></body></html>"
    };

    if let Some(sender) = state.sender.lock().expect("callback sender").take() {
        let _ = sender.send(outcome);
    }

    Html(body)
}

pub async fn login(
    http: &ClientWithMiddleware,
    issuer: &str,
    client_id: &str,
) -> Result<Credentials, OAuthError> {
    let metadata = discover(http, issuer).await?;
    let pkce = generate_pkce();
    let expected_state = random_state();

    let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .map_err(OAuthError::Listener)?;
    let port = listener.local_addr().map_err(OAuthError::Listener)?.port();
    let redirect_uri = format!("http://localhost:{port}/callback");

    let (sender, receiver) = oneshot::channel();
    let app = axum::Router::new()
        .route("/callback", get(callback_handler))
        .with_state(CallbackState {
            expected_state: expected_state.clone(),
            sender: Arc::new(Mutex::new(Some(sender))),
        });

    let server = tokio::spawn(async move { axum::serve(listener, app).await });

    let url = authorize_url(&metadata, client_id, &redirect_uri, &pkce, &expected_state);
    println!("opening {url}");
    open_browser(&url);

    let callback = match tokio::time::timeout(LOGIN_TIMEOUT, receiver).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_)) => Err(OAuthError::Callback(
            "the callback server stopped early".to_owned(),
        )),
        Err(_) => Err(OAuthError::Timeout),
    };

    server.abort();
    let callback = callback?;

    let form = [
        ("grant_type", "authorization_code"),
        ("code", callback.code.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("client_id", client_id),
        ("code_verifier", pkce.verifier.as_str()),
    ];

    let token = exchange(http, &metadata.token_endpoint, &form).await?;

    Ok(into_credentials(token, None, issuer, client_id))
}

pub async fn refresh(
    http: &ClientWithMiddleware,
    credentials: &Credentials,
) -> Result<Credentials, OAuthError> {
    let refresh_token = credentials
        .refresh_token
        .as_deref()
        .ok_or(OAuthError::NoRefreshToken)?;

    let metadata = discover(http, &credentials.issuer).await?;
    let form = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", credentials.client_id.as_str()),
    ];

    let token = exchange(http, &metadata.token_endpoint, &form).await?;

    Ok(into_credentials(
        token,
        credentials.refresh_token.clone(),
        &credentials.issuer,
        &credentials.client_id,
    ))
}

async fn exchange(
    http: &ClientWithMiddleware,
    token_endpoint: &str,
    form: &[(&str, &str)],
) -> Result<TokenResponse, OAuthError> {
    let response = http.post(token_endpoint).form(form).send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(OAuthError::Token(format!("{status}: {body}")));
    }

    Ok(response.json().await?)
}

fn into_credentials(
    token: TokenResponse,
    fallback_refresh_token: Option<String>,
    issuer: &str,
    client_id: &str,
) -> Credentials {
    let expires_at = token
        .expires_in
        .map(|seconds| Utc::now() + chrono::Duration::seconds(seconds));

    Credentials {
        access_token: token.access_token,
        refresh_token: token.refresh_token.or(fallback_refresh_token),
        expires_at,
        issuer: issuer.to_owned(),
        client_id: client_id.to_owned(),
    }
}

fn open_browser(url: &str) {
    let launcher = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    match Command::new(launcher).arg(url).spawn() {
        Ok(_) => {}
        Err(e) => println!("could not open a browser ({e}), visit the url above manually"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_the_rfc7636_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            challenge_for(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn a_generated_verifier_matches_its_own_challenge() {
        let pkce = generate_pkce();
        assert_eq!(pkce.challenge, challenge_for(&pkce.verifier));
        assert_eq!(pkce.verifier.len(), VERIFIER_LEN);
    }

    fn params(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn a_matching_callback_yields_the_code() {
        let callback =
            resolve_callback(&params(&[("code", "abc"), ("state", "xyz")]), "xyz").unwrap();
        assert_eq!(callback.code, "abc");
    }

    #[test]
    fn a_mismatched_state_is_rejected() {
        let error =
            resolve_callback(&params(&[("code", "abc"), ("state", "nope")]), "xyz").unwrap_err();
        assert!(matches!(error, OAuthError::StateMismatch));
    }

    #[test]
    fn a_provider_error_is_surfaced_with_its_description() {
        let error = resolve_callback(
            &params(&[("error", "access_denied"), ("error_description", "nope")]),
            "xyz",
        )
        .unwrap_err();

        assert!(matches!(error, OAuthError::Callback(m) if m == "access_denied: nope"));
    }

    #[test]
    fn a_callback_without_a_code_is_rejected() {
        let error = resolve_callback(&params(&[("state", "xyz")]), "xyz").unwrap_err();
        assert!(matches!(error, OAuthError::Callback(_)));
    }

    #[test]
    fn the_authorize_url_carries_pkce_and_encoded_scopes() {
        let metadata = ProviderMetadata {
            authorization_endpoint: "https://idm.example/authorize".to_owned(),
            token_endpoint: "https://idm.example/token".to_owned(),
        };
        let pkce = Pkce {
            verifier: "v".to_owned(),
            challenge: "c".to_owned(),
        };

        let url = authorize_url(
            &metadata,
            "home-gateway",
            "http://localhost:1234/callback",
            &pkce,
            "st",
        );

        assert!(url.starts_with("https://idm.example/authorize?"));
        assert!(url.contains("code_challenge=c"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A1234%2Fcallback"));
        assert!(url.contains("scope=openid%20email%20profile%20groups%20offline_access"));
    }
}
