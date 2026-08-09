use crate::http::{HttpCreationError, get_traced_http_client};
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;
use types::{UVXMLDocument, WeatherDetails};

pub mod types;

#[derive(thiserror::Error, Debug)]
pub enum WeatherAPIError {
    #[error(transparent)]
    Http(#[from] HttpCreationError),
    #[error("a http error occurred: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),
    #[error("a http error occurred: {0}")]
    Request(#[from] reqwest::Error),
    #[error("a xml error occurred: {0}")]
    Xml(#[from] quick_xml::de::DeError),
    #[error("location {0} not found in uv data")]
    LocationNotFound(String),
}

#[derive(Clone)]
pub struct WeatherAPI {
    http: ClientWithMiddleware,
}

impl WeatherAPI {
    const UV_LEVELS_XML: &str = "https://uvdata.arpansa.gov.au/xml/uvvalues.xml";
    const WEATHER_API_DETAILS: &str = "https://api.weather.bom.gov.au/v1/locations/{}/observations";
    pub const PERTH_NAME: &str = "per";
    pub const JANDAKOT_GEOCODE: &str = "qd63he";

    pub fn new() -> Result<Self, WeatherAPIError> {
        Ok(Self {
            http: get_traced_http_client()?,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_weather_details(
        &self,
        geocode: &str,
    ) -> Result<WeatherDetails, WeatherAPIError> {
        let weather_details = self
            .http
            .get(Self::WEATHER_API_DETAILS.replace("{}", geocode))
            .send()
            .await?
            .error_for_status()?
            .json::<WeatherDetails>()
            .await?;

        tracing::info!("fetched weather details");

        Ok(weather_details)
    }

    #[instrument(skip(self))]
    pub async fn get_uv_level(&self, name: &str) -> Result<f64, WeatherAPIError> {
        let uv_levels_xml = self
            .http
            .get(Self::UV_LEVELS_XML)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let uv_levels = quick_xml::de::from_str::<UVXMLDocument>(&uv_levels_xml)?;

        tracing::info!("fetched uv level data");

        uv_levels
            .location
            .into_iter()
            .find(|l| l.name == name)
            .map(|l| l.index)
            .ok_or_else(|| WeatherAPIError::LocationNotFound(name.to_owned()))
    }
}
