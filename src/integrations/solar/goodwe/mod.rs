use crate::cache::MemoryCache;
use crate::http::{HttpCreationError, wrap_client_in_middleware_no_tracing};
use crate::repo::SolarRepo;
use crate::settings::SolarSettings;
use axum::http::{HeaderMap, HeaderValue};
use base64::{Engine, prelude::BASE64_STANDARD};
use md5::Md5;
use reqwest::{
    Method,
    header::{ACCEPT, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT},
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tracing::instrument;
use types::{
    DeviceStatus, Envelope, LoginData, LoginRequest, SolarData, StationFlow, TelecountingGroup,
    numeric,
};

pub mod types;

const LOGIN_URL: &str = "https://semsplus.goodwe.com/web/sems/sems-user/api/v1/auth/cross-login";
const WEB_ORIGIN: &str = "https://semsplus.goodwe.com";
const BOOTSTRAP_TOKEN: &str =
    r#"{"uid":"","timestamp":0,"token":"","client":"semsPlusWeb","version":"","language":"en"}"#;
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

const DEVICE_STATUS_PATH: &str = "/sems-plant/api/stations/device/all-status";
const STATION_FLOW_PATH: &str = "/sems-plant/api/stations/flow";
const TELECOUNTING_TEMPLATE: &str = "/sems-plant/api/equipments/{serial_number}/telecounting";

const EXPIRED_TOKEN_CODES: [&str; 2] = ["100002", "C0602"];

const TOKEN_MAX_AGE_MINUTES: i64 = 60;
const TOKEN_MAX_AGE: Duration = Duration::from_secs(TOKEN_MAX_AGE_MINUTES as u64 * 60);
const SERIAL_NUMBER_MAX_AGE: Duration = Duration::from_secs(60 * 60);

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
    #[error("invalid header value: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("sems+ {operation} failed with code {code}: {description}")]
    Api {
        operation: &'static str,
        code: String,
        description: String,
    },
    #[error("sems+ station has no inverter")]
    NoInverter,
    #[error("sems+ response has no {0}")]
    MissingField(&'static str),
}

impl GoodWeSemsAPIError {
    fn is_expired_token(&self) -> bool {
        matches!(self, Self::Api { code, .. } if EXPIRED_TOKEN_CODES.contains(&code.as_str()))
    }
}

#[derive(Clone)]
pub struct GoodWeSemsAPI {
    db: Pool<Postgres>,
    username: String,
    password: String,
    powerstation_id: String,
    http: reqwest_middleware::ClientWithMiddleware,
    token: MemoryCache<(), LoginData>,
    serial_number: MemoryCache<(), String>,
}

impl GoodWeSemsAPI {
    pub fn new(db: Pool<Postgres>, settings: &SolarSettings, timeout: Duration) -> Option<Self> {
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
            .timeout(timeout)
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
            token: MemoryCache::builder("goodwe_token")
                .configure(|cache| cache.max_capacity(1).time_to_live(TOKEN_MAX_AGE))
                .build(),
            serial_number: MemoryCache::builder("goodwe_serial_number")
                .configure(|cache| cache.max_capacity(1).time_to_live(SERIAL_NUMBER_MAX_AGE))
                .build(),
        })
    }

    #[instrument(name = "goodwe.get_solar_data", skip(self))]
    pub async fn get_solar_data(&self) -> Result<SolarData, GoodWeSemsAPIError> {
        let login = self.get_new_or_cached_login_data().await?;

        match self.fetch_solar_data(&login).await {
            Err(e) if e.is_expired_token() => {
                tracing::info!("goodwe token was rejected, logging in again: {e}");
                self.token.invalidate(&()).await;
                let login = self.login_and_save().await?;
                self.fetch_solar_data(&login).await
            }
            result => result,
        }
    }

    async fn fetch_solar_data(&self, login: &LoginData) -> Result<SolarData, GoodWeSemsAPIError> {
        let serial_number = self.inverter_serial_number(login).await?;

        let flow_path = format!("{STATION_FLOW_PATH}?stationId={}", self.powerstation_id);
        let telecounting_path = format!(
            "{}?deviceType=INVERTER&pwId={}",
            TELECOUNTING_TEMPLATE.replace("{serial_number}", &serial_number),
            self.powerstation_id
        );

        let (flow, telecounting) = tokio::try_join!(
            self.get::<StationFlow>(login, "station flow", &flow_path, STATION_FLOW_PATH),
            self.get::<Vec<TelecountingGroup>>(
                login,
                "telecounting",
                &telecounting_path,
                TELECOUNTING_TEMPLATE
            ),
        )?;

        let current_kw = flow
            .p_ac
            .as_ref()
            .and_then(numeric)
            .ok_or(GoodWeSemsAPIError::MissingField("station flow pAc"))?;

        let counter = |code: &'static str| {
            telecounting
                .iter()
                .flat_map(|group| &group.factors)
                .find(|factor| factor.code == code)
                .and_then(|factor| numeric(&factor.data))
                .ok_or(GoodWeSemsAPIError::MissingField(code))
        };

        Ok(SolarData {
            current_w: current_kw * 1000.0,
            today_kwh: counter("proPvStatsToday")?,
            month_kwh: counter("proPvStatsMonth")?,
            total_kwh: counter("proPvStatsTotal")?,
            serial_number,
            flow,
            telecounting,
        })
    }

    async fn inverter_serial_number(
        &self,
        login: &LoginData,
    ) -> Result<String, GoodWeSemsAPIError> {
        if let Some(serial_number) = self.serial_number.get(&()).await {
            return Ok(serial_number);
        }

        let status = self
            .get::<DeviceStatus>(
                login,
                "device status",
                &format!("{DEVICE_STATUS_PATH}?stationId={}", self.powerstation_id),
                DEVICE_STATUS_PATH,
            )
            .await?;

        let serial_number = status
            .device_detail_list
            .into_iter()
            .filter(|group| group.device_type == "INVERTER")
            .flat_map(|group| group.status_detail_list)
            .flat_map(|group| group.sn_list)
            .next()
            .ok_or(GoodWeSemsAPIError::NoInverter)?;

        tracing::info!("using goodwe inverter {serial_number}");

        self.serial_number.insert((), serial_number.clone()).await;

        Ok(serial_number)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        login: &LoginData,
        operation: &'static str,
        path: &str,
        template: &'static str,
    ) -> Result<T, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(
                Method::GET,
                format!("{}{path}", login.api.trim_end_matches('/')),
            )
            .with_extension(crate::http::UrlTemplate(template))
            .header(
                "token",
                HeaderValue::from_str(&serde_json::to_string(login)?)?,
            )
            .header("X-Signature", signature(&login.uid, &login.token))
            .build()?;

        let envelope = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<Envelope<T>>()
            .await?;

        unwrap_envelope(operation, envelope)
    }

    #[instrument(name = "goodwe.get_new_or_cached_login_data", skip(self))]
    pub async fn get_new_or_cached_login_data(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        if let Some(login_data) = self.token.get(&()).await {
            return Ok(login_data);
        }

        let latest = SolarRepo::new(self.db.clone()).cached_token().await?;

        let Some(latest) = latest else {
            return self.login_and_save().await;
        };

        if (chrono::Utc::now() - latest.created_at).num_minutes() >= TOKEN_MAX_AGE_MINUTES {
            return self.login_and_save().await;
        }

        match serde_json::from_value::<LoginData>(latest.login_data) {
            Ok(login_data) => {
                tracing::debug!("using cached goodwe login data");
                self.token.insert((), login_data.clone()).await;
                Ok(login_data)
            }
            Err(e) => {
                tracing::warn!("stored goodwe login data is unreadable, logging in again: {e}");
                self.login_and_save().await
            }
        }
    }

    async fn login_and_save(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let login_data = self.login().await?;

        SolarRepo::new(self.db.clone())
            .save_cached_token(serde_json::to_value(&login_data)?)
            .await?;

        tracing::info!("saved goodwe login data");

        self.token.insert((), login_data.clone()).await;

        Ok(login_data)
    }

    #[instrument(name = "goodwe.login", skip(self))]
    pub async fn login(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(Method::POST, LOGIN_URL)
            .with_extension(crate::http::UrlTemplate(
                "/web/sems/sems-user/api/v1/auth/cross-login",
            ))
            .json(&LoginRequest {
                account: self.username.clone(),
                pwd: hash_password(&self.password),
                agreement: 1,
                is_chinese: false,
                is_local: false,
            })
            .header(ORIGIN, WEB_ORIGIN)
            .header(REFERER, format!("{WEB_ORIGIN}/"))
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .header("token", BOOTSTRAP_TOKEN)
            .header("X-Signature", signature("", ""))
            .build()?;

        let envelope = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<Envelope<LoginData>>()
            .await?;

        unwrap_envelope("login", envelope)
    }
}

fn unwrap_envelope<T>(
    operation: &'static str,
    envelope: Envelope<T>,
) -> Result<T, GoodWeSemsAPIError> {
    if !envelope.is_success() {
        return Err(GoodWeSemsAPIError::Api {
            operation,
            code: match envelope.code {
                serde_json::Value::String(code) => code,
                code => code.to_string(),
            },
            description: envelope.description.unwrap_or_default(),
        });
    }

    envelope
        .data
        .ok_or(GoodWeSemsAPIError::MissingField("data"))
}

fn hash_password(password: &str) -> String {
    BASE64_STANDARD.encode(hex::encode(Md5::digest(password.as_bytes())))
}

fn signature(uid: &str, token: &str) -> String {
    let epoch_ms = chrono::Utc::now().timestamp_millis();
    let digest = hex::encode(Sha256::digest(format!("{epoch_ms}@{uid}@{token}")));

    BASE64_STANDARD.encode(format!("{digest}@{epoch_ms}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_is_base64_of_md5_hex() {
        assert_eq!(
            hash_password("password"),
            "NWY0ZGNjM2I1YWE3NjVkNjFkODMyN2RlYjg4MmNmOTk="
        );
    }

    #[test]
    fn telecounting_and_flow_decode() {
        let flow: Envelope<StationFlow> = serde_json::from_str(
            r#"{"code":"00000","description":"成功","data":{"id":"s","status":"1","pAc":2.28,"refreshTime":"2026-09-23T13:56:00"}}"#,
        )
        .unwrap();
        assert!(flow.is_success());
        assert_eq!(
            flow.data.unwrap().p_ac.as_ref().and_then(numeric),
            Some(2.28)
        );

        let counters: Envelope<Vec<TelecountingGroup>> = serde_json::from_str(
            r#"{"code":"00000","data":[{"code":"telecounting_today","factors":[{"code":"proPvStatsToday","data":"12.34","unit":"kWh"}]}]}"#,
        )
        .unwrap();
        let groups = counters.data.unwrap();
        assert_eq!(numeric(&groups[0].factors[0].data), Some(12.34));
    }

    #[test]
    fn expired_token_codes_trigger_relogin() {
        let envelope: Envelope<serde_json::Value> =
            serde_json::from_str(r#"{"code":"C0602","description":"expired","data":null}"#)
                .unwrap();
        let error = unwrap_envelope("station flow", envelope).unwrap_err();
        assert!(error.is_expired_token());
    }
}
