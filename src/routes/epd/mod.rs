mod panel;
mod partial;
mod plan;
mod render;

use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
    auth::{Auth, scope::required},
    battery::BatteryChemistry,
    device_registry::DeviceRegistry,
    feature_flag::FeatureFlagClient,
    settings::SettingsContainer,
    types::{ApiState, AppError},
};
use axum::{
    Json,
    extract::{Query, State},
};
use chrono_tz::Australia::Perth;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::epd::{epd_flag_config, target_firmware_version};
use panel::{PACKED_FRAME_SIZE, crop_packed, packed_cache_key};
use partial::resolve_partial_window;
use plan::{ensure_packed_cached, render_plan};

pub use panel::PartialWindow;

const FIRMWARE_KEY_PREFIX: &str = "eink-display/firmware/";

const PREPARE_RENDER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

#[derive(Debug, Serialize, Deserialize, async_graphql::SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub image_url: Option<String>,
    pub image_hash: Option<String>,
    pub clear_screen: Option<bool>,
    pub firmware_url: Option<String>,
    pub firmware_version: Option<String>,
    pub partial: Option<PartialWindow>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DeviceReport<'a> {
    pub running_firmware_version: Option<&'a str>,
    pub current_image_hash: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfigRequest {
    pub device_id: String,
    pub battery_voltage: Option<f32>,
    pub is_charging: Option<bool>,
    pub battery_chemistry: Option<BatteryChemistry>,
    pub battery_kind: Option<String>,
    pub firmware_version: Option<String>,
    pub current_image_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceParams {
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageParams {
    pub device_id: String,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl ImageParams {
    fn window(&self) -> Result<Option<PartialWindow>, AppError> {
        let (Some(x), Some(y), Some(width), Some(height)) =
            (self.x, self.y, self.width, self.height)
        else {
            return Ok(None);
        };

        let window = PartialWindow {
            x,
            y,
            width,
            height,
        };

        if !window.is_valid() {
            tracing::warn!(
                device_id = %self.device_id,
                "rejecting misaligned partial window x={x} y={y} w={width} h={height}"
            );
            return Err(AppError::StatusCode(StatusCode::BAD_REQUEST));
        }

        Ok(Some(window))
    }
}

pub(crate) async fn build_epd_config(
    db: &sqlx::Pool<sqlx::Postgres>,
    s3: &crate::s3::S3,
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    settings: &SettingsContainer,
    device_id: &str,
    report: DeviceReport<'_>,
) -> EpdConfig {
    let DeviceReport {
        running_firmware_version,
        current_image_hash,
    } = report;

    #[cfg(debug_assertions)]
    let host = "http://192.168.0.149:8000/v1/epd";
    #[cfg(not(debug_assertions))]
    let host = "https://home.anurag.sh/v1/epd";

    let flag = epd_flag_config(feature_flag_client, devices, device_id).await;
    let configured_refresh = flag.resolve_refresh_interval(devices.eink_display(device_id));

    let Some(plan) = render_plan(
        db,
        s3,
        feature_flag_client,
        devices,
        settings,
        &flag,
        device_id,
    )
    .await
    else {
        tracing::warn!(device_id = %device_id, "no image to serve, skipping this cycle");
        return EpdConfig {
            refresh_interval_mins: Some(configured_refresh),
            image_url: None,
            image_hash: None,
            clear_screen: Some(flag.clear_screen),
            firmware_url: None,
            firmware_version: None,
            partial: None,
        };
    };

    let refresh_interval_mins = match plan.sleep {
        Some(sleep) => {
            let now = chrono::Utc::now().with_timezone(&Perth).time();
            sleep.minutes_until_end(now)
        }
        None => configured_refresh,
    };

    let firmware_update = match plan.sleep {
        Some(_) => None,
        None => target_firmware_version(&flag, devices, device_id)
            .filter(|target| Some(target.as_str()) != running_firmware_version),
    };

    let firmware_url = firmware_update
        .as_ref()
        .map(|_| format!("{host}/firmware?device_id={device_id}"));

    if !ensure_packed_cached(s3, &plan).await {
        return EpdConfig {
            refresh_interval_mins: Some(refresh_interval_mins),
            image_url: None,
            image_hash: None,
            clear_screen: Some(flag.clear_screen),
            firmware_url,
            firmware_version: firmware_update,
            partial: None,
        };
    }

    let partial = if flag.clear_screen || plan.sleep.is_some() {
        None
    } else {
        resolve_partial_window(
            db,
            s3,
            devices,
            &flag,
            device_id,
            current_image_hash,
            &plan.hash,
        )
        .await
    };

    let mut url = format!("{host}/image/{}?device_id={device_id}", plan.hash);

    if let Some(window) = partial {
        url.push_str(&format!(
            "&x={}&y={}&width={}&height={}",
            window.x, window.y, window.width, window.height
        ));
    }

    EpdConfig {
        refresh_interval_mins: Some(refresh_interval_mins),
        image_url: Some(url),
        image_hash: Some(plan.hash),
        clear_screen: Some(flag.clear_screen),
        firmware_url,
        firmware_version: firmware_update,
        partial,
    }
}

pub async fn image(
    State(ApiState { s3, .. }): State<ApiState>,
    Auth(auth): Auth,
    axum::extract::Path(hash): axum::extract::Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(&required::REST_EPD_READ)
        .map_err(AppError::StatusCode)?;

    let window = params.window()?;
    let key = packed_cache_key(hash);

    let Ok(packed) = s3.get_object(&key).await else {
        tracing::warn!(
            device_id = %params.device_id,
            key = %key,
            "pinned frame is not in the packed cache"
        );
        return Err(AppError::StatusCode(StatusCode::NOT_FOUND));
    };

    if packed.len() != PACKED_FRAME_SIZE {
        tracing::error!(
            device_id = %params.device_id,
            key = %key,
            len = packed.len(),
            "pinned frame is not {PACKED_FRAME_SIZE} bytes"
        );
        return Err(AppError::StatusCode(StatusCode::INTERNAL_SERVER_ERROR));
    }

    Ok(match window {
        Some(window) => crop_packed(&packed, window),
        None => packed,
    })
}

pub async fn config(
    State(ApiState {
        feature_flag_client,
        devices,
        db,
        s3,
        settings,
        ..
    }): State<ApiState>,
    Auth(auth): Auth,
    Json(request): Json<EpdConfigRequest>,
) -> Result<Json<EpdConfig>, AppError> {
    auth.require(&required::REST_EPD_READ)
        .map_err(AppError::StatusCode)?;

    let display = devices.eink_display(&request.device_id);
    let registered = display.is_some();
    let configured_mode = display.map(|display| display.mode.name());

    tracing::info!(
        device_id = %request.device_id,
        registered,
        ?configured_mode,
        current_image_hash = ?request.current_image_hash,
        battery_voltage = ?request.battery_voltage,
        is_charging = ?request.is_charging,
        battery_chemistry = ?request.battery_chemistry,
        battery_kind = ?request.battery_kind,
        firmware_version = ?request.firmware_version,
        "epd config requested"
    );

    if !registered {
        tracing::warn!(
            device_id = %request.device_id,
            "epd config request from unregistered display, add it to devices.yaml"
        );
    }

    report_to_actor(&request)?;

    let prepared =
        crate::actors::rpc::query(EInkDisplayActor::NAME, PREPARE_RENDER_TIMEOUT, |reply| {
            EInkDisplayMessage::PrepareRender {
                device_id: request.device_id.clone(),
                reply,
            }
        })
        .await;

    if let Err(e) = prepared {
        tracing::warn!(
            device_id = %request.device_id,
            "could not prepare a fresh render ({e}), serving the last one"
        );
    }

    let config = build_epd_config(
        &db,
        &s3,
        &feature_flag_client,
        &devices,
        &settings,
        &request.device_id,
        DeviceReport {
            running_firmware_version: request.firmware_version.as_deref(),
            current_image_hash: request.current_image_hash.as_deref(),
        },
    )
    .await;

    if let Some(wake_in_mins) = config.refresh_interval_mins {
        schedule_next_render(&request.device_id, wake_in_mins)?;
    }

    Ok(Json(config))
}

fn schedule_next_render(device_id: &str, wake_in_mins: u32) -> Result<(), AppError> {
    let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME.to_string()) else {
        tracing::warn!("eink display actor not found, not scheduling the next render");
        return Ok(());
    };

    actor.send_message(EInkDisplayMessage::ScheduleNextRender {
        device_id: device_id.to_owned(),
        wake_in_mins,
    })?;

    Ok(())
}

fn report_to_actor(request: &EpdConfigRequest) -> Result<(), AppError> {
    let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME.to_string()) else {
        tracing::warn!("eink display actor not found, dropping config request");
        return Ok(());
    };

    actor.send_message(EInkDisplayMessage::ConfigRequest {
        device_id: request.device_id.clone(),
    })?;

    let Some(voltage) = request.battery_voltage else {
        tracing::warn!(
            device_id = %request.device_id,
            "epd config request without a battery voltage, skipping battery report"
        );
        return Ok(());
    };

    actor.send_message(EInkDisplayMessage::BatteryReport {
        device_id: request.device_id.clone(),
        battery_voltage: voltage as f64,
        is_charging: request.is_charging,
        battery_chemistry: request.battery_chemistry,
        battery_kind: request.battery_kind.clone(),
    })?;

    Ok(())
}

pub async fn firmware(
    State(ApiState {
        s3,
        feature_flag_client,
        devices,
        ..
    }): State<ApiState>,
    Auth(auth): Auth,
    Query(params): Query<DeviceParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(&required::REST_EPD_READ)
        .map_err(AppError::StatusCode)?;

    let flag = epd_flag_config(&feature_flag_client, &devices, &params.device_id).await;

    let Some(version) = target_firmware_version(&flag, &devices, &params.device_id) else {
        tracing::warn!(
            device_id = %params.device_id,
            "firmware requested by an unregistered display"
        );
        return Err(AppError::StatusCode(StatusCode::NOT_FOUND));
    };

    let key = format!("{FIRMWARE_KEY_PREFIX}firmware_{version}.bin");

    tracing::info!(
        device_id = %params.device_id,
        version = %version,
        key = %key,
        "serving firmware"
    );

    Ok(s3.get_object(&key).await?)
}

pub async fn take_screenshot(Auth(auth): Auth) -> Result<StatusCode, AppError> {
    auth.require(&required::REST_EPD_WRITE)
        .map_err(AppError::StatusCode)?;

    let maybe_actor = ractor::registry::where_is(EInkDisplayActor::NAME.to_string());
    if let Some(actor) = maybe_actor {
        actor.send_message(EInkDisplayMessage::TakeScreenshot { device_id: None })?;
        Ok(StatusCode::CREATED)
    } else {
        Ok(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
