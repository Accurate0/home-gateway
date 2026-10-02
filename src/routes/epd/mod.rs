use crate::actors::system::rpc;
use crate::integrations::s3::S3;
use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    battery::BatteryChemistry,
    error::AppError,
    state::AppState,
};
use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::eink::EinkDisplayManager;
use crate::eink::panel::{PACKED_FRAME_SIZE, crop_packed, packed_cache_key};
use crate::eink::wake::{self, WakeContext, WakeReport};

pub use crate::eink::panel::PartialWindow;

const FIRMWARE_KEY_PREFIX: &str = "eink-display/firmware/";

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
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(hash): axum::extract::Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<bytes::Bytes, AppError> {
    auth.require(Resource::Epd, Action::Read)?;

    let window = params.window()?;

    if packed_cache_key(&hash).is_none() {
        tracing::warn!(
            device_id = %params.device_id,
            "rejected a frame request whose hash is not a frame hash"
        );
        return Err(AppError::StatusCode(StatusCode::BAD_REQUEST));
    }

    let Some(packed) = state
        .handles
        .expect::<EinkDisplayManager>()
        .packed_frame(&hash)
        .await
    else {
        tracing::warn!(
            device_id = %params.device_id,
            hash = %hash,
            "pinned frame is not in the packed cache"
        );
        return Err(AppError::StatusCode(StatusCode::NOT_FOUND));
    };

    if packed.len() != PACKED_FRAME_SIZE {
        tracing::error!(
            device_id = %params.device_id,
            hash = %hash,
            len = packed.len(),
            "pinned frame is not {PACKED_FRAME_SIZE} bytes"
        );
        return Err(AppError::StatusCode(StatusCode::INTERNAL_SERVER_ERROR));
    }

    Ok(match window {
        Some(window) => bytes::Bytes::from(crop_packed(&packed, window)),
        None => packed,
    })
}

pub async fn config(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(request): Json<EpdConfigRequest>,
) -> Result<Json<EpdConfig>, AppError> {
    auth.require(Resource::Epd, Action::Read)?;

    let eink = state.handles.expect::<EinkDisplayManager>();

    let context = WakeContext {
        devices: &state.devices,
        device_repo: state.repos.device(),
        eink,
        prepare_render_timeout: state.settings.eink_display.prepare_render_timeout(),
    };

    let report = WakeReport {
        device_id: &request.device_id,
        battery_voltage: request.battery_voltage,
        is_charging: request.is_charging,
        battery_chemistry: request.battery_chemistry,
        battery_kind: request.battery_kind.as_deref(),
        firmware_version: request.firmware_version.as_deref(),
        current_image_hash: request.current_image_hash.as_deref(),
    };

    let Some(resolved) = wake::begin(&context, &report).await? else {
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
        let next_wake_at = wake::next_wake_at(chrono::Utc::now(), wake_in_secs);

        eink.store_next_wake(&request.device_id, &resolved.name, next_wake_at)
            .await?;

        wake::schedule_next_render(&request.device_id, next_wake_at)?;
    }

    Ok(Json(config))
}

pub async fn firmware(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<DeviceParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(Resource::Epd, Action::Read)?;

    let Some(display) = state
        .handles
        .expect::<EinkDisplayManager>()
        .resolve(&params.device_id)
        .await
    else {
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

    Ok(state.handles.expect::<S3>().get_object(&key).await?)
}

pub async fn take_screenshot(auth: AuthContext) -> Result<StatusCode, AppError> {
    auth.require(Resource::Epd, Action::Write)?;

    rpc::cast(
        EInkDisplayActor::NAME,
        EInkDisplayMessage::TakeScreenshot { device_id: None },
    )
    .context("requesting screenshot")?;

    Ok(StatusCode::CREATED)
}
