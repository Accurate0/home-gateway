use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::jellyfin::types::Session;
use crate::settings::JellyfinSettings;

pub mod types;

pub const DEVICE_ID: &str = "home-gateway";

#[derive(thiserror::Error, Debug)]
pub enum JellyfinError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("jellyfin returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
}

#[derive(Clone)]
pub struct Jellyfin {
    base_url: String,
    api_key: String,
    client: ClientWithMiddleware,
}

impl Jellyfin {
    pub fn new(settings: &JellyfinSettings) -> Option<Self> {
        let base_url = settings.url.trim().trim_end_matches('/').to_owned();
        if base_url.is_empty() {
            tracing::warn!("jellyfin url is empty; disabling integration");
            return None;
        }

        let api_key = match settings.api_key.as_deref().map(str::trim) {
            Some(key) if !key.is_empty() => key.to_owned(),
            _ => {
                tracing::warn!(
                    "jellyfin is configured but JELLYFIN__API_KEY is missing; disabling integration"
                );
                return None;
            }
        };

        let client = match get_traced_http_client() {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("failed to build jellyfin http client: {e}");
                return None;
            }
        };

        tracing::info!("jellyfin integration enabled ({base_url})");
        Some(Self {
            base_url,
            api_key,
            client,
        })
    }

    pub fn ws_url(&self) -> String {
        let ws = if let Some(rest) = self.base_url.strip_prefix("https://") {
            format!("wss://{rest}")
        } else if let Some(rest) = self.base_url.strip_prefix("http://") {
            format!("ws://{rest}")
        } else {
            format!("ws://{}", self.base_url)
        };
        format!("{ws}/socket?api_key={}&deviceId={DEVICE_ID}", self.api_key)
    }

    #[instrument(skip(self))]
    pub async fn sessions(&self) -> Result<Vec<Session>, JellyfinError> {
        let url = format!("{}/Sessions", self.base_url);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let response = Self::error_for_status(response).await?;

        Ok(response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }

    fn auth_header(&self) -> String {
        format!(
            r#"MediaBrowser Client="home-gateway", Device="home-gateway", DeviceId="{DEVICE_ID}", Version="1", Token="{}""#,
            self.api_key
        )
    }

    async fn error_for_status(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, JellyfinError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        Err(JellyfinError::Status { status, body })
    }
}
