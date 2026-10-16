use cynic::{GraphQlResponse, QueryBuilder};
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::{UrlTemplate, get_traced_http_client};
use crate::settings::UnraidSettings;

pub mod lua;
mod query;
pub mod types;

use query::ArrayQuery;
use types::UnraidSnapshot;

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
}

#[derive(Clone)]
pub struct Unraid {
    url: String,
    api_key: String,
    client: ClientWithMiddleware,
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

        Ok(Self {
            url: format!("{base_url}/graphql"),
            api_key,
            client: get_traced_http_client(timeout)?,
        })
    }

    #[instrument(name = "unraid.snapshot", skip(self))]
    pub async fn snapshot(&self) -> Result<UnraidSnapshot, UnraidError> {
        let response = self
            .client
            .post(&self.url)
            .with_extension(UrlTemplate("/graphql"))
            .header("x-api-key", &self.api_key)
            .json(&ArrayQuery::build(()))
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            return Err(UnraidError::Status { status, body });
        }

        let response: GraphQlResponse<ArrayQuery> = response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?;

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

        data.array.try_into()
    }
}
