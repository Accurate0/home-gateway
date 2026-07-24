use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
    auth::{Auth, scope::required},
    device_registry::{DeviceRegistry, SleepWindow},
    feature_flag::FeatureFlagClient,
    types::{ApiState, AppError},
};
use ab_glyph::{FontRef, PxScale};
use axum::{
    Json,
    extract::{Query, State},
};
use chrono_tz::Australia::Perth;
use http::StatusCode;
use imageproc::drawing::{draw_text_mut, text_size};
use open_feature::EvaluationContext;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

const SLEEP_IMAGE_PREFIX: &str = "eink-display/sleep/";
const LABEL_FONT: &[u8] = include_bytes!("../../../assets/LiberationSans-Bold.ttf");

const EPD_FLAG: &str = "home-gateway-epd";

#[derive(Debug, Clone)]
struct EpdFlagConfig {
    clear_screen: bool,
    force_sleep: bool,
    refresh_interval: u32,
}

impl Default for EpdFlagConfig {
    fn default() -> Self {
        Self {
            clear_screen: false,
            force_sleep: false,
            refresh_interval: 15,
        }
    }
}

impl From<open_feature::StructValue> for EpdFlagConfig {
    fn from(value: open_feature::StructValue) -> Self {
        let default = EpdFlagConfig::default();
        Self {
            clear_screen: value
                .fields
                .get("clear_screen")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.clear_screen),
            force_sleep: value
                .fields
                .get("force_sleep")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.force_sleep),
            refresh_interval: value
                .fields
                .get("refresh_interval")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|n| n.max(0) as u32)
                .unwrap_or(default.refresh_interval),
        }
    }
}

async fn epd_flag_config(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    device_id: &str,
) -> EpdFlagConfig {
    let mut context =
        EvaluationContext::default().with_custom_field("device_id", device_id.to_string());
    if let Some(display) = devices.eink_display(device_id) {
        context = context.with_custom_field("device_name", display.name.clone());
    }

    match feature_flag_client.get_struct(EPD_FLAG, context).await {
        Ok(value) => {
            let config = EpdFlagConfig::from(value);
            tracing::info!("{EPD_FLAG} evaluated for {device_id}: {config:?}");
            config
        }
        Err(e) => {
            let fallback = EpdFlagConfig::default();
            tracing::error!(
                "error evaluating {EPD_FLAG} for {device_id}: {e:?}, using fallback: {fallback:?}"
            );
            fallback
        }
    }
}

fn active_sleep(
    devices: &DeviceRegistry,
    device_id: &str,
    force_sleep: bool,
) -> Option<SleepWindow> {
    let sleep = devices.eink_display(device_id).and_then(|d| d.sleep)?;
    let now = chrono::Utc::now().with_timezone(&Perth).time();
    if sleep.contains(now) || force_sleep {
        return Some(sleep);
    }
    None
}

async fn latest_image_key(
    db: &sqlx::Pool<sqlx::Postgres>,
    device_id: &str,
) -> Result<String, AppError> {
    sqlx::query_scalar!(
        "SELECT image_key FROM eink_display WHERE device_id = $1",
        device_id
    )
    .fetch_optional(db)
    .await?
    .flatten()
    .ok_or_else(|| AppError::Error(anyhow::anyhow!("no rendered image for device {device_id}")))
}

fn draw_sleep_label(img: &mut image::RgbImage, label: &str) {
    let Ok(font) = FontRef::try_from_slice(LABEL_FONT) else {
        tracing::warn!("failed to load label font, skipping sleep label");
        return;
    };

    let scale = PxScale::from(120.0);
    let (text_w, text_h) = text_size(scale, &font, label);
    let margin = 48i32;

    let (img_w, img_h) = img.dimensions();
    let x = img_w as i32 - text_w as i32 - margin;
    let y = img_h as i32 - text_h as i32 - margin;

    draw_text_mut(img, image::Rgb([0, 0, 0]), x, y, scale, &font, label);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub image_url: Option<String>,
    pub clear_screen: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfigRequest {
    pub device_id: String,
    pub battery_voltage: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct LatestParams {
    pub device_id: String,
}

pub async fn config(
    State(ApiState {
        feature_flag_client,
        devices,
        ..
    }): State<ApiState>,
    Auth(auth): Auth,
    Json(request): Json<EpdConfigRequest>,
) -> Result<Json<EpdConfig>, AppError> {
    auth.require(&required::REST_EPD_READ)
        .map_err(AppError::StatusCode)?;

    if let Some(voltage) = request.battery_voltage {
        if let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME.to_string()) {
            actor.send_message(EInkDisplayMessage::BatteryReport {
                device_id: request.device_id.clone(),
                battery_voltage: voltage as f64,
            })?;
        } else {
            tracing::warn!("eink display actor not found, dropping battery report");
        }
    }

    #[cfg(debug_assertions)]
    let base = "http://192.168.0.149:8000/v1/epd/latest";
    #[cfg(not(debug_assertions))]
    let base = "https://home.anurag.sh/v1/epd/latest";

    let flag = epd_flag_config(&feature_flag_client, &devices, &request.device_id).await;

    if let Some(sleep) = active_sleep(&devices, &request.device_id, flag.force_sleep) {
        let now = chrono::Utc::now().with_timezone(&Perth).time();
        return Ok(Json(EpdConfig {
            refresh_interval_mins: Some(sleep.minutes_until_end(now)),
            image_url: Some(format!("{base}?device_id={}", request.device_id)),
            clear_screen: Some(false),
        }));
    }

    Ok(Json(EpdConfig {
        refresh_interval_mins: Some(flag.refresh_interval),
        image_url: Some(format!("{base}?device_id={}", request.device_id)),
        clear_screen: Some(flag.clear_screen),
    }))
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

pub async fn latest(
    State(ApiState {
        s3,
        db,
        feature_flag_client,
        devices,
        ..
    }): State<ApiState>,
    Auth(auth): Auth,
    Query(params): Query<LatestParams>,
) -> Result<Vec<u8>, AppError> {
    auth.require(&required::REST_EPD_READ)
        .map_err(AppError::StatusCode)?;

    let flag = epd_flag_config(&feature_flag_client, &devices, &params.device_id).await;

    let mut sleep_label = None;
    let image_key = match active_sleep(&devices, &params.device_id, flag.force_sleep) {
        Some(sleep) => {
            let images = s3.list_objects(SLEEP_IMAGE_PREFIX).await?;
            let picked = images.choose(&mut rand::rng()).cloned();
            match picked {
                Some(image_key) => {
                    sleep_label = Some(format!("zzz till {}", sleep.end.format("%H:%M")));
                    image_key
                }
                None => {
                    tracing::warn!(
                        "no sleep images under {SLEEP_IMAGE_PREFIX}, serving latest render"
                    );
                    latest_image_key(&db, &params.device_id).await?
                }
            }
        }
        None => latest_image_key(&db, &params.device_id).await?,
    };

    let image_response = s3.get_object(&image_key).await?;

    let output_packed = tokio::task::spawn_blocking(move || {
        let mut img = image::load_from_memory(&image_response)?.to_rgb8();
        let (width, height) = img.dimensions();

        if let Some(label) = &sleep_label {
            draw_sleep_label(&mut img, label);
        }

        if width == 1600 && height == 1200 {
            img = image::imageops::rotate90(&img);
        } else if width != 1200 || height != 1600 {
            return Err(anyhow::anyhow!(
                "Image dimensions must be 1600x1200 (will be rotated) or 1200x1600"
            ));
        }

        let (width, height) = img.dimensions();

        // Convert to floating point for error diffusion
        let mut buffer: Vec<f32> = Vec::with_capacity((width * height * 3) as usize);
        for pixel in img.pixels() {
            buffer.push(pixel[0] as f32);
            buffer.push(pixel[1] as f32);
            buffer.push(pixel[2] as f32);
        }

        let mut output_packed = Vec::with_capacity((width * height / 2) as usize);

        // Palette: Black, White, Yellow, Red, Blue, Green
        // Indices: 0, 1, 2, 3, 5, 6
        let palette = [
            (0.0, 0.0, 0.0, 0u8),       // Black
            (255.0, 255.0, 255.0, 1u8), // White
            (255.0, 255.0, 0.0, 2u8),   // Yellow
            (255.0, 0.0, 0.0, 3u8),     // Red
            (0.0, 0.0, 255.0, 5u8),     // Blue
            (0.0, 255.0, 0.0, 6u8),     // Green
        ];

        for y in 0..height {
            for x in (0..width).step_by(2) {
                // Process pixel 1 (high nibble)
                let idx1 = process_pixel(&mut buffer, width, height, x, y, &palette);

                // Process pixel 2 (low nibble)
                let idx2 = process_pixel(&mut buffer, width, height, x + 1, y, &palette);

                output_packed.push((idx1 << 4) | idx2);
            }
        }
        Ok(output_packed)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Join error: {}", e))??;

    Ok(output_packed)
}

fn process_pixel(
    buffer: &mut [f32],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    palette: &[(f32, f32, f32, u8)],
) -> u8 {
    let index = ((y * width + x) * 3) as usize;
    let r = buffer[index];
    let g = buffer[index + 1];
    let b = buffer[index + 2];

    // Find closest color
    let mut min_dist = f32::MAX;
    let mut closest_idx = 0;
    let mut closest_color = (0.0, 0.0, 0.0);

    for &(pr, pg, pb, pidx) in palette {
        let dr = r - pr;
        let dg = g - pg;
        let db = b - pb;
        let dist = dr * dr + dg * dg + db * db;
        if dist < min_dist {
            min_dist = dist;
            closest_idx = pidx;
            closest_color = (pr, pg, pb);
        }
    }

    let (pr, pg, pb) = closest_color;
    let err_r = r - pr;
    let err_g = g - pg;
    let err_b = b - pb;

    // Distribute error
    // (x+1, y) 7/16
    if x + 1 < width {
        add_error(buffer, width, x + 1, y, err_r, err_g, err_b, 7.0 / 16.0);
    }
    // (x-1, y+1) 3/16
    if x > 0 && y + 1 < height {
        add_error(buffer, width, x - 1, y + 1, err_r, err_g, err_b, 3.0 / 16.0);
    }
    // (x, y+1) 5/16
    if y + 1 < height {
        add_error(buffer, width, x, y + 1, err_r, err_g, err_b, 5.0 / 16.0);
    }
    // (x+1, y+1) 1/16
    if x + 1 < width && y + 1 < height {
        add_error(buffer, width, x + 1, y + 1, err_r, err_g, err_b, 1.0 / 16.0);
    }

    closest_idx
}

#[allow(clippy::too_many_arguments)]
fn add_error(
    buffer: &mut [f32],
    width: u32,
    x: u32,
    y: u32,
    er: f32,
    eg: f32,
    eb: f32,
    factor: f32,
) {
    let index = ((y * width + x) * 3) as usize;
    buffer[index] += er * factor;
    buffer[index + 1] += eg * factor;
    buffer[index + 2] += eb * factor;
}
