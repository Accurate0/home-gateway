use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    battery::BatteryChemistry,
    error::AppError,
    state::ApiState,
};
use axum::{
    Json,
    extract::{Query, State},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::eink::EinkDisplayManager;
use crate::eink::panel::{PACKED_FRAME_SIZE, crop_packed, packed_cache_key};

pub use crate::eink::panel::PartialWindow;

const FIRMWARE_KEY_PREFIX: &str = "eink-display/firmware/";

const PREPARE_RENDER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

#[derive(Debug, Serialize, Deserialize, async_graphql::SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub refresh_interval_secs: Option<u32>,
    pub image_url: Option<String>,
    pub image_hash: Option<String>,
    pub clear_screen: Option<bool>,
    pub firmware_url: Option<String>,
    pub firmware_version: Option<String>,
    pub partial: Option<PartialWindow>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeviceReport<'a> {
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

pub async fn image(
    State(ApiState { s3, .. }): State<ApiState>,
    Auth(auth): Auth,
    axum::extract::Path(hash): axum::extract::Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(&Scope::new(Resource::Epd, Action::Read))
        .map_err(AppError::StatusCode)?;

    let window = params.window()?;

    let Some(key) = packed_cache_key(&hash) else {
        tracing::warn!(
            device_id = %params.device_id,
            "rejected a frame request whose hash is not a frame hash"
        );
        return Err(AppError::StatusCode(StatusCode::BAD_REQUEST));
    };

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
    State(ApiState { eink, devices, .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(request): Json<EpdConfigRequest>,
) -> Result<Json<EpdConfig>, AppError> {
    auth.require(&Scope::new(Resource::Epd, Action::Read))
        .map_err(AppError::StatusCode)?;

    let display = devices.eink_display(&request.device_id);
    let registered = display.is_some();
    let configured_mode = display.map(|display| display.mode.name());
    let wake_drift_secs = wake_drift_secs(&eink, &request.device_id).await;

    tracing::info!(
        device_id = %request.device_id,
        registered,
        ?configured_mode,
        ?wake_drift_secs,
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

    let prepared = crate::actors::system::rpc::query(
        EInkDisplayActor::NAME,
        PREPARE_RENDER_TIMEOUT,
        |reply| EInkDisplayMessage::PrepareRender {
            device_id: request.device_id.clone(),
            reply,
        },
    )
    .await;

    if let Err(e) = prepared {
        tracing::warn!(
            device_id = %request.device_id,
            "could not prepare a fresh render ({e}), serving the last one"
        );
    }

    let Some(resolved) = eink.resolve(&request.device_id).await else {
        return Err(AppError::StatusCode(StatusCode::NOT_FOUND));
    };

    let config = eink
        .epd_config(
            &resolved,
            DeviceReport {
                running_firmware_version: request.firmware_version.as_deref(),
                current_image_hash: request.current_image_hash.as_deref(),
            },
        )
        .await;

    if let Some(wake_in_secs) = config.refresh_interval_secs {
        schedule_next_render(&request.device_id, wake_in_secs)?;
    }

    Ok(Json(config))
}

async fn wake_drift_secs(eink: &EinkDisplayManager, device_id: &str) -> Option<i64> {
    let next_wake_at = eink
        .stored_next_wake(device_id)
        .await
        .inspect_err(|_| tracing::warn!(device_id, "could not read the stored wake"))
        .ok()
        .flatten()?;

    let drift = (chrono::Utc::now() - next_wake_at).num_seconds();

    crate::metrics::record_eink_wake_drift(device_id.to_owned(), drift as f64);

    Some(drift)
}

fn schedule_next_render(device_id: &str, wake_in_secs: u32) -> Result<(), AppError> {
    let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME) else {
        tracing::warn!("eink display actor not found, not scheduling the next render");
        return Ok(());
    };

    actor.send_message(EInkDisplayMessage::ScheduleNextRender {
        device_id: device_id.to_owned(),
        wake_in_secs,
    })?;

    Ok(())
}

fn report_to_actor(request: &EpdConfigRequest) -> Result<(), AppError> {
    let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME) else {
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
    State(ApiState { s3, eink, .. }): State<ApiState>,
    Auth(auth): Auth,
    Query(params): Query<DeviceParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(&Scope::new(Resource::Epd, Action::Read))
        .map_err(AppError::StatusCode)?;

    let Some(display) = eink.resolve(&params.device_id).await else {
        tracing::warn!(
            device_id = %params.device_id,
            "firmware requested by an unregistered display"
        );
        return Err(AppError::StatusCode(StatusCode::NOT_FOUND));
    };

    let version = display.firmware_version;
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
    auth.require(&Scope::new(Resource::Epd, Action::Write))
        .map_err(AppError::StatusCode)?;

    let maybe_actor = ractor::registry::where_is(EInkDisplayActor::NAME);
    if let Some(actor) = maybe_actor {
        actor.send_message(EInkDisplayMessage::TakeScreenshot { device_id: None })?;
        Ok(StatusCode::CREATED)
    } else {
        Ok(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
