use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::settings::HolidaySettings;

pub mod ics;
pub mod types;

pub use types::Holiday;

#[derive(thiserror::Error, Debug)]
pub enum HolidaysError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("holiday calendar could not be parsed: {0}")]
    Parse(String),
    #[error("holiday calendar returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
}

#[derive(Clone)]
pub struct Holidays {
    client: ClientWithMiddleware,
    url: String,
}

impl Holidays {
    pub fn new(
        settings: &HolidaySettings,
        timeout: std::time::Duration,
    ) -> Result<Self, HolidaysError> {
        tracing::info!(
            "holidays integration enabled for [{}]",
            settings.regions.join(", ")
        );

        Ok(Self {
            client: get_traced_http_client(timeout)?,
            url: settings.url.clone(),
        })
    }

    #[instrument(skip(self), err)]
    pub async fn fetch(&self) -> Result<Vec<Holiday>, HolidaysError> {
        let response = self
            .client
            .get(&self.url)
            .with_extension(crate::http::UrlTemplate("/calendar/ical"))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(HolidaysError::Status { status, body });
        }

        let body = response
            .text()
            .await
            .map_err(reqwest_middleware::Error::from)?;

        ics::parse(&body).map_err(HolidaysError::Parse)
    }
}
