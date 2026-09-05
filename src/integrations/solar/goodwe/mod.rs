use crate::http::{HttpCreationError, wrap_client_in_middleware_no_tracing};
use crate::repo::SolarRepo;
use crate::settings::SolarSettings;
use axum::http::{HeaderMap, HeaderValue};
use base64::{Engine, prelude::BASE64_STANDARD};
use reqwest::{
    Method,
    header::{ACCEPT, CONTENT_TYPE},
};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use tracing::instrument;
use types::{
    LoginData, LoginRequest, LoginResponse, PlantDetailsByPowerStationIdResponse, SavedSolarData,
};

pub mod types;

const LOGIN_URL: &str = "https://www.semsportal.com/api/v2/Common/CrossLogin";
const GET_POWERSTATION_DETAILS_URL: &str =
    "https://au.semsportal.com/api/v3/PowerStation/GetPlantDetailByPowerstationId";

const BOOTSTRAP_TOKEN: &str = "eyJ1aWQiOiIiLCJ0aW1lc3RhbXAiOjAsInRva2VuIjoiIiwiY2xpZW50Ijoid2ViIiwidmVyc2lvbiI6IiIsImxhbmd1YWdlIjoiZW4ifQ==";

const TOKEN_MAX_AGE_MINUTES: i64 = 5;

#[derive(thiserror::Error, Debug)]
pub enum GoodWeSemsAPIError {
    #[error(transparent)]
    Http(#[from] HttpCreationError),
    #[error("a http error occurred: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),
    #[error("a http error occurred: {0}")]
    Request(#[from] reqwest::Error),
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not decode stored data: {0}")]
    Decode(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct GoodWeSemsAPI {
    db: Pool<Postgres>,
    username: String,
    password: String,
    powerstation_id: String,
    http: reqwest_middleware::ClientWithMiddleware,
}

impl GoodWeSemsAPI {
    pub fn new(db: Pool<Postgres>, settings: &SolarSettings) -> Option<Self> {
        let username = settings.goodwe_username.as_deref().map(str::trim)?;
        let password = settings.goodwe_password.as_deref().map(str::trim)?;
        let powerstation_id = settings.goodwe_powerstation_id.as_deref().map(str::trim)?;

        if username.is_empty() || password.is_empty() || powerstation_id.is_empty() {
            tracing::warn!("goodwe credentials are empty; disabling solar integration");
            return None;
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = match reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .map_err(HttpCreationError::from)
            .and_then(wrap_client_in_middleware_no_tracing)
        {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("failed to build goodwe http client: {e}");
                return None;
            }
        };

        Some(Self {
            db,
            username: username.to_owned(),
            password: password.to_owned(),
            powerstation_id: powerstation_id.to_owned(),
            http: client,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_latest_saved_solar_data(
        &self,
    ) -> Result<Option<SavedSolarData>, GoodWeSemsAPIError> {
        let solar_data = SolarRepo::new(self.db.clone()).latest().await?;

        solar_data
            .map(|row| {
                Ok(SavedSolarData {
                    raw_data: serde_json::from_value(row.raw_data)?,
                    temperature: row.temperature,
                    uv_level: row.uv_level,
                })
            })
            .transpose()
    }

    #[instrument(skip(self))]
    pub async fn get_solar_data(
        &self,
        login: LoginData,
    ) -> Result<PlantDetailsByPowerStationIdResponse, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(Method::POST, GET_POWERSTATION_DETAILS_URL)
            .form(&{
                let mut map = HashMap::new();
                map.insert("powerStationId", &self.powerstation_id);
                map
            })
            .header(
                "token",
                BASE64_STANDARD.encode(serde_json::to_string(&login)?),
            )
            .build()?;

        let response = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<PlantDetailsByPowerStationIdResponse>()
            .await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn get_new_or_cached_login_data(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let latest = SolarRepo::new(self.db.clone()).cached_token().await?;

        let Some(latest) = latest else {
            return self.login_and_save().await;
        };

        if (chrono::Utc::now() - latest.created_at).num_minutes() > TOKEN_MAX_AGE_MINUTES {
            return self.login_and_save().await;
        }

        match serde_json::from_value::<LoginData>(latest.login_data) {
            Ok(login_data) => {
                tracing::info!("using cached goodwe login data");
                Ok(login_data)
            }
            Err(e) => {
                tracing::warn!("stored goodwe login data is unreadable, logging in again: {e}");
                self.login_and_save().await
            }
        }
    }

    async fn login_and_save(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let response = self.login().await?;
        let login_data = serde_json::to_value(&response.data)?;

        SolarRepo::new(self.db.clone())
            .save_cached_token(login_data)
            .await?;

        tracing::info!("saved goodwe login data");

        Ok(response.data)
    }

    #[instrument(skip(self))]
    pub async fn login(&self) -> Result<LoginResponse, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(Method::POST, LOGIN_URL)
            .json(&LoginRequest {
                account: self.username.clone(),
                pwd: self.password.clone(),
            })
            .header("token", HeaderValue::from_static(BOOTSTRAP_TOKEN))
            .build()?;

        let response = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<LoginResponse>()
            .await?;

        Ok(response)
    }
}
