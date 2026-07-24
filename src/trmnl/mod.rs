use tracing::instrument;

use crate::{
    http::wrap_client_in_middleware_no_tracing,
    trmnl::types::{TrmnlDevice, TrmnlDevicesResponse},
};

pub mod types;

pub struct Trmnl {
    client: reqwest_middleware::ClientWithMiddleware,
    api_key: String,
    base_url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TrmnlError {
    #[error(transparent)]
    HttpMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl Trmnl {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: wrap_client_in_middleware_no_tracing(reqwest::Client::new()).unwrap(),
        }
    }

    #[instrument(skip(self))]
    pub async fn list_devices(&self) -> Result<Vec<TrmnlDevice>, TrmnlError> {
        let url = format!("{}/api/devices", self.base_url);
        let resp = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<TrmnlDevicesResponse>()
            .await?;

        Ok(resp.data)
    }
}
