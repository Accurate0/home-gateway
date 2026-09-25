use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::jellyfin::types::Session;
use crate::settings::JellyfinSettings;

pub mod types;
pub mod websocket;

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
    #[error("integrations.jellyfin.api_key is missing")]
    MissingApiKey,
}

#[derive(Clone)]
pub struct Jellyfin {
    base_url: String,
    api_key: String,
    client: ClientWithMiddleware,
}

impl Jellyfin {
    pub fn new(
        settings: &JellyfinSettings,
        timeout: std::time::Duration,
    ) -> Result<Self, JellyfinError> {
        let base_url = settings.url.trim().trim_end_matches('/').to_owned();

        let api_key = settings
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .ok_or(JellyfinError::MissingApiKey)?
            .to_owned();

        tracing::info!("jellyfin integration enabled ({base_url})");

        Ok(Self {
            base_url,
            api_key,
            client: get_traced_http_client(timeout)?,
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

    #[instrument(name = "jellyfin.sessions", skip(self))]
    pub async fn sessions(&self) -> Result<Vec<Session>, JellyfinError> {
        let url = format!("{}/Sessions", self.base_url);

        let response = self
            .client
            .get(url)
            .with_extension(crate::http::UrlTemplate("/Sessions"))
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
