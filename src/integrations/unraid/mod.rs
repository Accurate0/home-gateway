use cynic::{GraphQlResponse, Operation, QueryBuilder};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio_tungstenite::Connector;
use tracing::instrument;

use crate::http::{HttpCreationError, TimeTrace, UrlTemplate};
use crate::settings::UnraidSettings;
use crate::tls::unverified_client_config;

pub mod lua;
mod query;
pub mod types;
pub mod watch;
mod websocket;

use query::{ArrayQuery, UnreadNotificationsQuery, UnreadNotificationsVariables};
use types::{UnraidNotification, UnraidPoll};

#[cynic::schema("unraid")]
mod schema {}

#[derive(thiserror::Error, Debug)]
pub enum UnraidError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("unraid returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("unraid returned no data: {0}")]
    NoData(String),
    #[error("unraid returned an unexpected {field}: `{value}`")]
    Unexpected { field: &'static str, value: String },
    #[error("integrations.unraid.api_key is missing")]
    MissingApiKey,
    #[error(transparent)]
    Tls(#[from] rustls::Error),
}

#[derive(Clone)]
pub struct Unraid {
    url: String,
    api_key: String,
    client: ClientWithMiddleware,
    ws_connector: Option<Connector>,
}

impl Unraid {
    pub fn new(
        settings: &UnraidSettings,
        timeout: std::time::Duration,
    ) -> Result<Self, UnraidError> {
        let base_url = settings.url.trim().trim_end_matches('/');

        let api_key = settings
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .ok_or(UnraidError::MissingApiKey)?
            .to_owned();

        tracing::info!("unraid integration enabled ({base_url})");

        let verify_certificates = settings.certificate_verification.is_enabled();

        if !verify_certificates {
            tracing::warn!("unraid certificate verification is disabled");
        }

        let client = reqwest::ClientBuilder::new()
            .timeout(timeout)
            .tls_danger_accept_invalid_certs(!verify_certificates)
            .build()
            .map_err(HttpCreationError::from)?;

        let ws_connector = if verify_certificates {
            None
        } else {
            Some(Connector::Rustls(unverified_client_config()?))
        };

        Ok(Self {
            url: format!("{base_url}/graphql"),
            api_key,
            client: ClientBuilder::new(client)
                .with(TracingMiddleware::<TimeTrace>::new())
                .build(),
            ws_connector,
        })
    }

    pub fn ws_connector(&self) -> Option<Connector> {
        self.ws_connector.clone()
    }

    pub fn ws_url(&self) -> String {
        match self.url.split_once("://") {
            Some(("https", rest)) => format!("wss://{rest}"),
            Some((_, rest)) => format!("ws://{rest}"),
            None => format!("ws://{}", self.url),
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    #[instrument(name = "unraid.snapshot", skip(self))]
    pub async fn snapshot(&self) -> Result<UnraidPoll, UnraidError> {
        self.post::<ArrayQuery, _>(&ArrayQuery::build(()))
            .await?
            .try_into()
    }

    #[instrument(name = "unraid.unread_notifications", skip(self))]
    pub async fn unread_notifications(
        &self,
        limit: i32,
    ) -> Result<Vec<UnraidNotification>, UnraidError> {
        let operation = UnreadNotificationsQuery::build(UnreadNotificationsVariables { limit });

        let notifications = self
            .post::<UnreadNotificationsQuery, _>(&operation)
            .await?
            .notifications
            .list
            .into_iter()
            .map(UnraidNotification::from)
            .collect();

        Ok(notifications)
    }

    async fn post<T, V>(&self, operation: &Operation<T, V>) -> Result<T, UnraidError>
    where
        T: DeserializeOwned,
        V: Serialize,
    {
        let response = self
            .client
            .post(&self.url)
            .with_extension(UrlTemplate("/graphql"))
            .header("x-api-key", &self.api_key)
            .json(operation)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            return Err(UnraidError::Status { status, body });
        }

        let response: GraphQlResponse<T> = response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?;

        response_data(response)
    }
}

fn response_data<T>(response: GraphQlResponse<T>) -> Result<T, UnraidError> {
    let errors = response
        .errors
        .unwrap_or_default()
        .into_iter()
        .map(|error| error.message)
        .collect::<Vec<_>>()
        .join("; ");

    let Some(data) = response.data else {
        return Err(UnraidError::NoData(errors));
    };

    if !errors.is_empty() {
        tracing::warn!("unraid answered with partial errors: {errors}");
    }

    Ok(data)
}
