use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;

use crate::http::get_traced_http_client;

pub const URL_ENV: &str = "HOME_ASSISTANT_URL";
pub const TOKEN_ENV: &str = "HOME_ASSISTANT_TOKEN";

#[derive(thiserror::Error, Debug)]
pub enum HomeAssistantError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("home assistant returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("invalid service `{0}`, expected `domain.service`")]
    InvalidService(String),
}

#[derive(Clone)]
pub struct HomeAssistant {
    base_url: String,
    token: String,
    client: ClientWithMiddleware,
}

impl HomeAssistant {
    pub fn from_env() -> Option<Self> {
        let base_url = match std::env::var(URL_ENV) {
            Ok(url) if !url.trim().is_empty() => url.trim().trim_end_matches('/').to_owned(),
            _ => return None,
        };

        let token = match std::env::var(TOKEN_ENV) {
            Ok(token) if !token.trim().is_empty() => token,
            _ => {
                tracing::warn!(
                    "{URL_ENV} is set but {TOKEN_ENV} is missing; disabling integration"
                );
                return None;
            }
        };

        let client = match get_traced_http_client() {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("failed to build home assistant http client: {e}");
                return None;
            }
        };

        tracing::info!("home assistant integration enabled ({base_url})");
        Some(Self {
            base_url,
            token,
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
        format!("{ws}/api/websocket")
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        data: Value,
    ) -> Result<(), HomeAssistantError> {
        let url = format!("{}/api/services/{domain}/{service}", self.base_url);
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.token)
            .json(&data)
            .send()
            .await?;

        Self::error_for_status(response).await.map(|_| ())
    }

    #[allow(unused)]
    pub async fn get_states(&self) -> Result<Value, HomeAssistantError> {
        let url = format!("{}/api/states", self.base_url);
        let response = self.client.get(url).bearer_auth(&self.token).send().await?;
        let response = Self::error_for_status(response).await?;
        Ok(response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }

    #[allow(unused)]
    pub async fn get_state(&self, entity_id: &str) -> Result<Value, HomeAssistantError> {
        let url = format!("{}/api/states/{entity_id}", self.base_url);
        let response = self.client.get(url).bearer_auth(&self.token).send().await?;
        let response = Self::error_for_status(response).await?;
        Ok(response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }

    async fn error_for_status(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, HomeAssistantError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        Err(HomeAssistantError::Status { status, body })
    }
}
