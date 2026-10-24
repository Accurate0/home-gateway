use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::{UrlTemplate, get_traced_http_client};
use crate::integrations::immich::types::{ImmichAlbum, ImmichAsset, RandomSearch};
use crate::settings::ImmichSettings;

pub mod types;

const API_KEY_HEADER: &str = "x-api-key";

#[derive(thiserror::Error, Debug)]
pub enum ImmichError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("immich returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("integrations.immich.api_key is missing")]
    MissingApiKey,
    #[error("immich album `{0}` not found")]
    AlbumNotFound(String),
    #[error("immich album name `{0}` matches {1} albums")]
    AmbiguousAlbum(String, usize),
    #[error("image too large: {0} bytes")]
    TooLarge(usize),
}

#[derive(Clone)]
pub struct Immich {
    base_url: String,
    api_key: String,
    candidates: u32,
    client: ClientWithMiddleware,
}

impl Immich {
    const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;

    pub fn new(
        settings: &ImmichSettings,
        timeout: std::time::Duration,
    ) -> Result<Self, ImmichError> {
        let base_url = settings.url.trim().trim_end_matches('/').to_owned();

        let api_key = settings
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .ok_or(ImmichError::MissingApiKey)?
            .to_owned();

        tracing::info!("immich integration enabled ({base_url})");

        Ok(Self {
            base_url,
            api_key,
            candidates: settings.candidates,
            client: get_traced_http_client(timeout)?,
        })
    }

    pub fn candidates(&self) -> u32 {
        self.candidates
    }

    #[instrument(name = "immich.album_id", skip(self))]
    pub async fn album_id(&self, name: &str) -> Result<String, ImmichError> {
        let response = self
            .client
            .get(format!("{}/api/albums", self.base_url))
            .with_extension(UrlTemplate("/api/albums"))
            .header(API_KEY_HEADER, &self.api_key)
            .send()
            .await?;

        let albums: Vec<ImmichAlbum> = Self::error_for_status(response)
            .await?
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?;

        let mut matching = albums
            .into_iter()
            .filter(|album| album.album_name == name)
            .collect::<Vec<_>>();

        match matching.len() {
            0 => Err(ImmichError::AlbumNotFound(name.to_owned())),
            1 => Ok(matching.remove(0).id),
            count => Err(ImmichError::AmbiguousAlbum(name.to_owned(), count)),
        }
    }

    #[instrument(name = "immich.random_assets", skip(self))]
    pub async fn random_assets(
        &self,
        album_id: &str,
        size: u32,
    ) -> Result<Vec<ImmichAsset>, ImmichError> {
        let search = RandomSearch {
            album_ids: [album_id],
            kind: "IMAGE",
            size,
            with_exif: true,
        };

        let response = self
            .client
            .post(format!("{}/api/search/random", self.base_url))
            .with_extension(UrlTemplate("/api/search/random"))
            .header(API_KEY_HEADER, &self.api_key)
            .json(&search)
            .send()
            .await?;

        Ok(Self::error_for_status(response)
            .await?
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }

    #[instrument(name = "immich.download_original", skip(self))]
    pub async fn download_original(&self, asset_id: &str) -> Result<Vec<u8>, ImmichError> {
        let response = self
            .client
            .get(format!("{}/api/assets/{asset_id}/original", self.base_url))
            .with_extension(UrlTemplate("/api/assets/{id}/original"))
            .header(API_KEY_HEADER, &self.api_key)
            .send()
            .await?;

        let bytes = Self::error_for_status(response)
            .await?
            .bytes()
            .await
            .map_err(reqwest_middleware::Error::from)?;

        if bytes.len() > Self::MAX_IMAGE_BYTES {
            return Err(ImmichError::TooLarge(bytes.len()));
        }

        Ok(bytes.to_vec())
    }

    async fn error_for_status(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, ImmichError> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await.unwrap_or_default();

        Err(ImmichError::Status { status, body })
    }
}
