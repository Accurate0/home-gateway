use crate::{
    actors::eink_display::{EInkDisplayActor, EInkDisplayMessage},
    auth::{Auth, scope::required},
    battery::BatteryChemistry,
    device_registry::{DeviceRegistry, EinkDisplaySettings, SleepWindow},
    feature_flag::FeatureFlagClient,
    settings::{
        Album, DashboardView, EinkGlobalSettings, EinkMode, PaletteColor, SettingsContainer,
    },
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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SLEEP_IMAGE_PREFIX: &str = "eink-display/sleep/";
const FIRMWARE_KEY_PREFIX: &str = "eink-display/firmware/";
const LABEL_FONT: &[u8] = include_bytes!("../../../assets/LiberationSans-Bold.ttf");

const RENDER_PIPELINE_VERSION: u32 = 1;

const EPD_FLAG: &str = "home-gateway-epd";
const EPD_NEW_DITHERING_FLAG: &str = "home-gateway-epd-new-dithering";
const EPD_DITHERING_CONFIG_FLAG: &str = "home-gateway-epd-dithering-config";

#[derive(Debug, Clone)]
pub(crate) struct EpdFlagConfig {
    clear_screen: bool,
    force_sleep: bool,
    refresh_interval: u32,
    mode: Option<EinkMode>,
    dashboard_view: Option<String>,
    album: Option<String>,
    firmware_version: Option<String>,
    partial_refresh: Option<bool>,
}

impl Default for EpdFlagConfig {
    fn default() -> Self {
        Self {
            clear_screen: false,
            force_sleep: false,
            refresh_interval: 15,
            mode: None,
            dashboard_view: None,
            album: None,
            firmware_version: None,
            partial_refresh: None,
        }
    }
}

impl From<open_feature::StructValue> for EpdFlagConfig {
    fn from(value: open_feature::StructValue) -> Self {
        let default = EpdFlagConfig::default();
        let string_field = |key: &str| {
            value
                .fields
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        };
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
            mode: string_field("mode").and_then(|s| match s.as_str() {
                "dashboard" => Some(EinkMode::Dashboard),
                "album" => Some(EinkMode::Album),
                other => {
                    tracing::warn!("unknown epd mode `{other}` in flag, ignoring");
                    None
                }
            }),
            dashboard_view: string_field("dashboard_view"),
            album: string_field("album"),
            firmware_version: string_field("firmware_version"),
            partial_refresh: value
                .fields
                .get("partial_refresh")
                .and_then(|v| v.as_bool()),
        }
    }
}

impl EpdFlagConfig {
    pub(crate) fn resolve_mode(&self, display: &EinkDisplaySettings) -> EinkMode {
        self.mode.unwrap_or_else(|| display.mode.name())
    }

    pub(crate) fn resolve_view<'a>(
        &self,
        global: &'a EinkGlobalSettings,
        display: &EinkDisplaySettings,
    ) -> Option<&'a DashboardView> {
        let name = self
            .dashboard_view
            .clone()
            .or_else(|| display.mode.view().map(|v| v.to_owned()));
        match name {
            Some(name) => global.view(&name).or_else(|| {
                tracing::warn!("dashboard view `{name}` not configured, ignoring");
                None
            }),
            None => None,
        }
    }

    pub(crate) fn resolve_album(
        &self,
        global: &EinkGlobalSettings,
        display: &EinkDisplaySettings,
    ) -> Album {
        let name = self
            .album
            .clone()
            .or_else(|| display.mode.album().map(|a| a.to_owned()));
        match name {
            Some(name) => global.album(&name).cloned().unwrap_or_else(|| {
                tracing::warn!("album `{name}` not configured, using default prefix");
                global.default_album()
            }),
            None => global.default_album(),
        }
    }
}

pub(crate) async fn epd_flag_config(
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

pub(crate) async fn epd_new_dithering_enabled(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    device_id: &str,
) -> bool {
    let mut context =
        EvaluationContext::default().with_custom_field("device_id", device_id.to_string());
    if let Some(display) = devices.eink_display(device_id) {
        context = context.with_custom_field("device_name", display.name.clone());
    }

    feature_flag_client
        .is_feature_enabled(EPD_NEW_DITHERING_FLAG, false, context)
        .await
}

pub(crate) async fn epd_palette(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    base: &[PaletteColor],
    device_id: &str,
) -> Vec<(f32, f32, f32, u8)> {
    let mut context =
        EvaluationContext::default().with_custom_field("device_id", device_id.to_string());
    if let Some(display) = devices.eink_display(device_id) {
        context = context.with_custom_field("device_name", display.name.clone());
    }

    let config = feature_flag_client
        .get_struct(EPD_DITHERING_CONFIG_FLAG, context)
        .await
        .ok();

    base.iter()
        .map(|color| {
            let (r, g, b) = config
                .as_ref()
                .and_then(|c| c.fields.get(color.name))
                .and_then(|v| v.as_struct())
                .map(|s| {
                    let channel = |key: &str, default: f32| {
                        s.fields
                            .get(key)
                            .and_then(|v| {
                                v.as_i64()
                                    .map(|n| n as f32)
                                    .or_else(|| v.as_f64().map(|f| f as f32))
                            })
                            .map(|n| n.clamp(0.0, 255.0))
                            .unwrap_or(default)
                    };
                    (
                        channel("r", color.r),
                        channel("g", color.g),
                        channel("b", color.b),
                    )
                })
                .unwrap_or((color.r, color.g, color.b));
            (r, g, b, color.index)
        })
        .collect()
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

async fn latest_image(
    db: &sqlx::Pool<sqlx::Postgres>,
    device_id: &str,
) -> Result<(String, String), AppError> {
    let row = sqlx::query!(
        "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
        device_id
    )
    .fetch_optional(db)
    .await?;

    let image_key = row
        .as_ref()
        .and_then(|row| row.image_key.clone())
        .ok_or_else(|| {
            AppError::Error(anyhow::anyhow!("no rendered image for device {device_id}"))
        })?;

    let content_hash = row
        .and_then(|row| row.image_content_hash)
        .unwrap_or_default();

    Ok((image_key, content_hash))
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, async_graphql::SimpleObject,
)]
#[graphql(rename_fields = "camelCase")]
pub struct PartialWindow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

const PANEL_WIDTH: u32 = 1200;
const PANEL_HEIGHT: u32 = 1600;
const PANEL_HALF_WIDTH: u32 = PANEL_WIDTH / 2;
const WINDOW_X_ALIGN: u32 = 4;
const WINDOW_MIN_WIDTH: u32 = 16;

impl PartialWindow {
    fn is_valid(&self) -> bool {
        self.width >= WINDOW_MIN_WIDTH
            && self.height >= 2
            && self.x.is_multiple_of(WINDOW_X_ALIGN)
            && self.width.is_multiple_of(WINDOW_X_ALIGN)
            && self.y.is_multiple_of(2)
            && self.height.is_multiple_of(2)
            && self.x + self.width <= PANEL_WIDTH
            && self.y + self.height <= PANEL_HEIGHT
    }

    fn area_pct(&self) -> u32 {
        (self.width as u64 * self.height as u64 * 100 / (PANEL_WIDTH as u64 * PANEL_HEIGHT as u64))
            as u32
    }
}

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

fn dirty_window(previous: &[u8], next: &[u8]) -> Option<PartialWindow> {
    if previous.len() != PACKED_FRAME_SIZE || next.len() != PACKED_FRAME_SIZE {
        return None;
    }

    let mut min_byte = PACKED_ROW_BYTES;
    let mut max_byte = 0usize;
    let mut min_row = PACKED_FRAME_HEIGHT;
    let mut max_row = 0usize;

    for row in 0..PACKED_FRAME_HEIGHT {
        let start = row * PACKED_ROW_BYTES;
        let previous_row = &previous[start..start + PACKED_ROW_BYTES];
        let next_row = &next[start..start + PACKED_ROW_BYTES];

        if previous_row == next_row {
            continue;
        }

        let first = previous_row
            .iter()
            .zip(next_row)
            .position(|(a, b)| a != b)
            .unwrap_or(0);
        let last = previous_row
            .iter()
            .zip(next_row)
            .rposition(|(a, b)| a != b)
            .unwrap_or(PACKED_ROW_BYTES - 1);

        min_byte = min_byte.min(first);
        max_byte = max_byte.max(last);
        min_row = min_row.min(row);
        max_row = max_row.max(row);
    }

    if min_row > max_row {
        return None;
    }

    let x0 = (min_byte as u32 * 2) / WINDOW_X_ALIGN * WINDOW_X_ALIGN;
    let x1 = ((max_byte as u32 + 1) * 2).div_ceil(WINDOW_X_ALIGN) * WINDOW_X_ALIGN;
    let y0 = min_row as u32 & !1;
    let y1 = (max_row as u32 + 1).div_ceil(2) * 2;

    let mut window = PartialWindow {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    };

    if window.width < WINDOW_MIN_WIDTH {
        window.width = WINDOW_MIN_WIDTH;
        window.x = window.x.min(PANEL_WIDTH - WINDOW_MIN_WIDTH);
    }

    if window.x < PANEL_HALF_WIDTH && window.x + window.width > PANEL_HALF_WIDTH {
        let left = PANEL_HALF_WIDTH - window.x;
        let right = window.x + window.width - PANEL_HALF_WIDTH;

        if left < WINDOW_MIN_WIDTH {
            window.x = PANEL_HALF_WIDTH - WINDOW_MIN_WIDTH;
            window.width = right + WINDOW_MIN_WIDTH;
        }

        if right < WINDOW_MIN_WIDTH {
            window.width = PANEL_HALF_WIDTH - window.x + WINDOW_MIN_WIDTH;
        }
    }

    window.is_valid().then_some(window)
}

fn render_hash(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }

    hex::encode(hasher.finalize())
}

struct RenderPlan {
    image_key: String,
    sleep: Option<SleepWindow>,
    sleep_label: Option<String>,
    new_dithering: bool,
    palette: Vec<(f32, f32, f32, u8)>,
    hash: String,
}

fn window_start_date(sleep: &SleepWindow, now: chrono::NaiveDateTime) -> chrono::NaiveDate {
    if now.time() >= sleep.start {
        now.date()
    } else {
        now.date().pred_opt().unwrap_or(now.date())
    }
}

fn pick_sleep_image(images: &[String], seed: &str) -> Option<String> {
    if images.is_empty() {
        return None;
    }

    let mut sorted = images.to_vec();
    sorted.sort();

    let digest = Sha256::digest(seed.as_bytes());
    let index = u64::from_be_bytes(digest[..8].try_into().ok()?) % sorted.len() as u64;

    sorted.into_iter().nth(index as usize)
}

async fn render_plan(
    db: &sqlx::Pool<sqlx::Postgres>,
    s3: &crate::s3::S3,
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    settings: &SettingsContainer,
    flag: &EpdFlagConfig,
    device_id: &str,
) -> Option<RenderPlan> {
    let new_dithering = epd_new_dithering_enabled(feature_flag_client, devices, device_id).await;
    let palette = epd_palette(
        feature_flag_client,
        devices,
        &settings.eink_display.palette,
        device_id,
    )
    .await;

    let palette_hash = palette
        .iter()
        .map(|(r, g, b, index)| format!("{r}:{g}:{b}:{index}"))
        .collect::<Vec<_>>()
        .join(",");

    let sleep = active_sleep(devices, device_id, flag.force_sleep);
    let now = chrono::Utc::now().with_timezone(&Perth).naive_local();

    let (image_key, content_hash, sleep_label) = match sleep {
        Some(sleep) => {
            let images = s3
                .list_objects(SLEEP_IMAGE_PREFIX)
                .await
                .unwrap_or_default();
            let seed = format!("{}/{}", sleep.end, window_start_date(&sleep, now));

            match pick_sleep_image(&images, &seed) {
                Some(image_key) => {
                    let label = format!("zzz till {}", sleep.end.format("%H:%M"));
                    (image_key.clone(), image_key, Some(label))
                }
                None => {
                    tracing::warn!(
                        "no sleep images under {SLEEP_IMAGE_PREFIX}, serving the latest render"
                    );
                    let (image_key, content_hash) = latest_image(db, device_id).await.ok()?;
                    (image_key, content_hash, None)
                }
            }
        }
        None => {
            let (image_key, content_hash) = latest_image(db, device_id).await.ok()?;
            (image_key, content_hash, None)
        }
    };

    let hash = render_hash(&[
        &RENDER_PIPELINE_VERSION.to_string(),
        &image_key,
        &content_hash,
        &new_dithering.to_string(),
        &palette_hash,
        &flag.clear_screen.to_string(),
        sleep_label.as_deref().unwrap_or(""),
    ]);

    Some(RenderPlan {
        image_key,
        sleep,
        sleep_label,
        new_dithering,
        palette,
        hash,
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DeviceReport<'a> {
    pub running_firmware_version: Option<&'a str>,
    pub current_image_hash: Option<&'a str>,
    pub prepare_render: bool,
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
        prepare_render,
    } = report;

    #[cfg(debug_assertions)]
    let host = "http://192.168.0.149:8000/v1/epd";
    #[cfg(not(debug_assertions))]
    let host = "https://home.anurag.sh/v1/epd";

    let flag = epd_flag_config(feature_flag_client, devices, device_id).await;

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
            refresh_interval_mins: Some(flag.refresh_interval),
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
        None => flag.refresh_interval,
    };

    let firmware_update = match plan.sleep {
        Some(_) => None,
        None => target_firmware_version(&flag, devices, device_id)
            .filter(|target| Some(target.as_str()) != running_firmware_version),
    };

    let prepared = !prepare_render || ensure_packed_cached(s3, &plan).await;

    if !prepared {
        return EpdConfig {
            refresh_interval_mins: Some(refresh_interval_mins),
            image_url: None,
            image_hash: None,
            clear_screen: Some(flag.clear_screen),
            firmware_url: firmware_update
                .as_ref()
                .map(|_| format!("{host}/firmware?device_id={device_id}")),
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
        firmware_url: firmware_update
            .as_ref()
            .map(|_| format!("{host}/firmware?device_id={device_id}")),
        firmware_version: firmware_update,
        partial,
    }
}

async fn ensure_packed_cached(s3: &crate::s3::S3, plan: &RenderPlan) -> bool {
    let key = packed_cache_key(plan.hash.clone());

    match s3.get_object_metadata(&key).await {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(key = %key, "failed to check the packed cache: {e}");
            return false;
        }
    }

    let result = render_and_cache(
        s3,
        &plan.image_key,
        plan.sleep_label.clone(),
        plan.new_dithering,
        &plan.palette,
        &key,
    )
    .await;

    match result {
        Ok(_) => true,
        Err(_) => {
            tracing::warn!(key = %key, "failed to prepare the packed frame");
            false
        }
    }
}

async fn resolve_partial_window(
    db: &sqlx::Pool<sqlx::Postgres>,
    s3: &crate::s3::S3,
    devices: &DeviceRegistry,
    flag: &EpdFlagConfig,
    device_id: &str,
    current_image_hash: Option<&str>,
    new_hash: &str,
) -> Option<PartialWindow> {
    let display = devices.eink_display(device_id)?;
    let policy = display.partial;

    if !flag.partial_refresh.unwrap_or(policy.enabled) {
        return None;
    }

    let current_image_hash = current_image_hash?;

    if current_image_hash == new_hash {
        return None;
    }

    let consecutive = partial_refresh_count(db, device_id).await;
    if consecutive >= policy.max_consecutive {
        tracing::info!(
            device_id = %device_id,
            consecutive,
            "forcing a full refresh to clear accumulated ghosting"
        );
        reset_partial_refresh_count(db, device_id).await;
        return None;
    }

    let previous = s3
        .get_object(&packed_cache_key(current_image_hash.to_owned()))
        .await
        .ok();
    let next = s3
        .get_object(&packed_cache_key(new_hash.to_owned()))
        .await
        .ok();

    let window = match (previous.as_deref(), next.as_deref()) {
        (Some(previous), Some(next)) => dirty_window(previous, next),
        _ => None,
    };

    let Some(window) = window else {
        reset_partial_refresh_count(db, device_id).await;
        return None;
    };

    let area_pct = window.area_pct();

    if area_pct > policy.max_area_pct {
        tracing::info!(
            device_id = %device_id,
            area_pct,
            "dirty region too large for a partial refresh"
        );
        reset_partial_refresh_count(db, device_id).await;
        return None;
    }

    tracing::info!(
        device_id = %device_id,
        area_pct,
        x = window.x,
        y = window.y,
        width = window.width,
        height = window.height,
        "serving a partial refresh"
    );

    increment_partial_refresh_count(db, device_id).await;

    Some(window)
}

async fn partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) -> i32 {
    let result = sqlx::query_scalar!(
        "SELECT partial_refresh_count FROM eink_display WHERE device_id = $1",
        device_id
    )
    .fetch_optional(db)
    .await;

    match result {
        Ok(count) => count.unwrap_or(0),
        Err(e) => {
            tracing::warn!(device_id = %device_id, "failed to read partial refresh count: {e}");
            0
        }
    }
}

async fn increment_partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) {
    let result = sqlx::query!(
        "UPDATE eink_display SET partial_refresh_count = partial_refresh_count + 1 WHERE device_id = $1",
        device_id
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::warn!(device_id = %device_id, "failed to increment partial refresh count: {e}");
    }
}

async fn reset_partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) {
    let result = sqlx::query!(
        "UPDATE eink_display SET partial_refresh_count = 0 WHERE device_id = $1",
        device_id
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::warn!(device_id = %device_id, "failed to reset partial refresh count: {e}");
    }
}

fn target_firmware_version(
    flag: &EpdFlagConfig,
    devices: &DeviceRegistry,
    device_id: &str,
) -> Option<String> {
    if let Some(override_version) = &flag.firmware_version {
        return Some(override_version.clone());
    }

    devices
        .eink_display(device_id)
        .map(|display| display.firmware_version.clone())
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

    let registered = devices.eink_display(&request.device_id).is_some();

    tracing::info!(
        device_id = %request.device_id,
        registered,
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

    if let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME.to_string()) {
        actor.send_message(EInkDisplayMessage::ConfigRequest {
            device_id: request.device_id.clone(),
        })?;

        if let Some(voltage) = request.battery_voltage {
            actor.send_message(EInkDisplayMessage::BatteryReport {
                device_id: request.device_id.clone(),
                battery_voltage: voltage as f64,
                is_charging: request.is_charging,
                battery_chemistry: request.battery_chemistry,
                battery_kind: request.battery_kind.clone(),
            })?;
        } else {
            tracing::warn!(
                device_id = %request.device_id,
                "epd config request without a battery voltage, skipping battery report"
            );
        }
    } else {
        tracing::warn!("eink display actor not found, dropping config request");
    }

    Ok(Json(
        build_epd_config(
            &db,
            &s3,
            &feature_flag_client,
            &devices,
            &settings,
            &request.device_id,
            DeviceReport {
                running_firmware_version: request.firmware_version.as_deref(),
                current_image_hash: request.current_image_hash.as_deref(),
                prepare_render: true,
            },
        )
        .await,
    ))
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

const PACKED_CACHE_PREFIX: &str = "eink-display/packed/";
pub const PACKED_ROW_BYTES: usize = 600;
pub const PACKED_FRAME_HEIGHT: usize = 1600;
pub const PACKED_FRAME_SIZE: usize = PACKED_ROW_BYTES * PACKED_FRAME_HEIGHT;

fn packed_cache_key(hash: String) -> String {
    format!("{PACKED_CACHE_PREFIX}{hash}.bin")
}

async fn render_and_cache(
    s3: &crate::s3::S3,
    image_key: &str,
    sleep_label: Option<String>,
    new_dithering: bool,
    palette: &[(f32, f32, f32, u8)],
    cache_key: &str,
) -> Result<Vec<u8>, AppError> {
    let image_response = s3.get_object(image_key).await?;
    let packed =
        render_packed(image_response, sleep_label, new_dithering, palette.to_vec()).await?;

    if let Err(e) = s3.put_object(cache_key, &packed, None).await {
        tracing::warn!(key = %cache_key, "failed to cache packed frame: {e}");
    }

    Ok(packed)
}

fn crop_packed(packed: &[u8], window: PartialWindow) -> Vec<u8> {
    let row_offset = (window.x / 2) as usize;
    let row_count = (window.width / 2) as usize;

    let mut cropped = Vec::with_capacity(row_count * window.height as usize);
    for row in window.y..window.y + window.height {
        let start = row as usize * PACKED_ROW_BYTES + row_offset;
        cropped.extend_from_slice(&packed[start..start + row_count]);
    }

    cropped
}

async fn render_packed(
    image_response: Vec<u8>,
    sleep_label: Option<String>,
    new_dithering: bool,
    palette: Vec<(f32, f32, f32, u8)>,
) -> Result<Vec<u8>, AppError> {
    let packed = tokio::task::spawn_blocking(move || {
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

        let indices = if new_dithering {
            dither_improved(&buffer, width, height, &palette)
        } else {
            let mut indices = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    indices[(y * width + x) as usize] =
                        process_pixel(&mut buffer, width, height, x, y, &palette);
                }
            }
            indices
        };

        let mut output_packed = Vec::with_capacity((width * height / 2) as usize);
        for y in 0..height {
            for x in (0..width).step_by(2) {
                let idx1 = indices[(y * width + x) as usize];
                let idx2 = indices[(y * width + x + 1) as usize];
                output_packed.push((idx1 << 4) | idx2);
            }
        }
        Ok(output_packed)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Join error: {}", e))??;

    Ok(packed)
}

fn srgb_to_linear(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn dither_improved(
    buffer: &[f32],
    width: u32,
    height: u32,
    palette: &[(f32, f32, f32, u8)],
) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;

    let mut lin: Vec<f32> = Vec::with_capacity(w * h * 3);
    for &v in buffer.iter() {
        lin.push(srgb_to_linear(v));
    }

    let palette_lin: Vec<(f32, f32, f32, u8)> = palette
        .iter()
        .map(|&(r, g, b, idx)| (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), idx))
        .collect();

    let palette_srgb: Vec<(f32, f32, f32)> = palette
        .iter()
        .map(|&(r, g, b, _)| (r / 255.0, g / 255.0, b / 255.0))
        .collect();

    let mut indices = vec![0u8; w * h];

    for y in 0..h {
        let left_to_right = y % 2 == 0;
        let xs: Vec<usize> = if left_to_right {
            (0..w).collect()
        } else {
            (0..w).rev().collect()
        };
        let dir: isize = if left_to_right { 1 } else { -1 };

        for &x in &xs {
            let base = (y * w + x) * 3;
            let r = lin[base];
            let g = lin[base + 1];
            let b = lin[base + 2];

            let sr = linear_to_srgb(r);
            let sg = linear_to_srgb(g);
            let sb = linear_to_srgb(b);

            let mut min_dist = f32::MAX;
            let mut closest = 0usize;
            for (i, &(pr, pg, pb)) in palette_srgb.iter().enumerate() {
                let dr = sr - pr;
                let dg = sg - pg;
                let db = sb - pb;
                let dist = dr * dr + dg * dg + db * db;
                if dist < min_dist {
                    min_dist = dist;
                    closest = i;
                }
            }

            let (lr, lg, lb, pidx) = palette_lin[closest];
            indices[y * w + x] = pidx;

            let er = r - lr;
            let eg = g - lg;
            let eb = b - lb;

            let mut spread = |nx: isize, ny: isize, factor: f32| {
                if nx < 0 || nx >= width as isize || ny < 0 || ny >= height as isize {
                    return;
                }
                let nbase = ((ny as usize) * w + nx as usize) * 3;
                lin[nbase] = (lin[nbase] + er * factor).clamp(0.0, 1.0);
                lin[nbase + 1] = (lin[nbase + 1] + eg * factor).clamp(0.0, 1.0);
                lin[nbase + 2] = (lin[nbase + 2] + eb * factor).clamp(0.0, 1.0);
            };

            let xi = x as isize;
            let yi = y as isize;
            spread(xi + dir, yi, 7.0 / 16.0);
            spread(xi - dir, yi + 1, 3.0 / 16.0);
            spread(xi, yi + 1, 5.0 / 16.0);
            spread(xi + dir, yi + 1, 1.0 / 16.0);
        }
    }

    indices
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_hash_is_stable() {
        assert_eq!(
            render_hash(&["1", "key.png"]),
            render_hash(&["1", "key.png"])
        );
    }

    #[test]
    fn render_hash_changes_with_inputs() {
        let base = render_hash(&["1", "key.png", "false"]);

        assert_ne!(base, render_hash(&["2", "key.png", "false"]));
        assert_ne!(base, render_hash(&["1", "other.png", "false"]));
        assert_ne!(base, render_hash(&["1", "key.png", "true"]));
    }

    #[test]
    fn render_hash_separates_parts() {
        assert_ne!(render_hash(&["ab", "c"]), render_hash(&["a", "bc"]));
    }

    fn sleep_images() -> Vec<String> {
        (0..8).map(|i| format!("sleep/{i}.png")).collect()
    }

    #[test]
    fn a_sleep_pick_is_stable_for_the_same_seed() {
        let images = sleep_images();
        assert_eq!(
            pick_sleep_image(&images, "08:00/2026-08-04"),
            pick_sleep_image(&images, "08:00/2026-08-04")
        );
    }

    #[test]
    fn a_sleep_pick_ignores_listing_order() {
        let images = sleep_images();
        let mut reversed = images.clone();
        reversed.reverse();

        assert_eq!(
            pick_sleep_image(&images, "08:00/2026-08-04"),
            pick_sleep_image(&reversed, "08:00/2026-08-04")
        );
    }

    #[test]
    fn a_sleep_pick_varies_across_nights() {
        let images = sleep_images();
        let picks = (1..=10)
            .map(|day| pick_sleep_image(&images, &format!("08:00/2026-08-{day:02}")))
            .collect::<std::collections::HashSet<_>>();

        assert!(picks.len() > 1, "every night picked the same image");
    }

    #[test]
    fn an_empty_sleep_listing_picks_nothing() {
        assert_eq!(pick_sleep_image(&[], "08:00/2026-08-04"), None);
    }

    #[test]
    fn a_sleep_window_that_straddles_midnight_keeps_one_start_date() {
        let sleep = SleepWindow {
            start: chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        };

        let evening = chrono::NaiveDate::from_ymd_opt(2026, 8, 4)
            .unwrap()
            .and_hms_opt(23, 0, 0)
            .unwrap();
        let morning = chrono::NaiveDate::from_ymd_opt(2026, 8, 5)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();

        assert_eq!(
            window_start_date(&sleep, evening),
            window_start_date(&sleep, morning)
        );
    }

    fn blank_frame() -> Vec<u8> {
        vec![0x11; PACKED_FRAME_SIZE]
    }

    fn set_pixel(frame: &mut [u8], x: usize, y: usize) {
        frame[y * PACKED_ROW_BYTES + x / 2] = 0x00;
    }

    #[test]
    fn an_unchanged_frame_has_no_dirty_window() {
        assert_eq!(dirty_window(&blank_frame(), &blank_frame()), None);
    }

    #[test]
    fn a_wrongly_sized_frame_has_no_dirty_window() {
        assert_eq!(dirty_window(&[0u8; 8], &[1u8; 8]), None);
    }

    #[test]
    fn a_single_changed_pixel_yields_the_minimum_window() {
        let previous = blank_frame();
        let mut next = previous.clone();
        set_pixel(&mut next, 400, 500);

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert!(window.is_valid());
        assert_eq!(window.width, WINDOW_MIN_WIDTH);
        assert_eq!(window.height, 2);
        assert_eq!(window.y, 500);
        assert!(window.x <= 400 && 400 < window.x + window.width);
    }

    #[test]
    fn a_change_straddling_the_controller_split_covers_both_halves() {
        let previous = blank_frame();
        let mut next = previous.clone();
        set_pixel(&mut next, 598, 100);
        set_pixel(&mut next, 602, 100);

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert!(window.is_valid());
        assert!(window.x + WINDOW_MIN_WIDTH <= PANEL_HALF_WIDTH);
        assert!(window.x + window.width >= PANEL_HALF_WIDTH + WINDOW_MIN_WIDTH);
    }

    #[test]
    fn a_full_frame_change_covers_the_whole_panel() {
        let previous = blank_frame();
        let next = vec![0x00; PACKED_FRAME_SIZE];

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert_eq!(
            window,
            PartialWindow {
                x: 0,
                y: 0,
                width: PANEL_WIDTH,
                height: PANEL_HEIGHT,
            }
        );
        assert_eq!(window.area_pct(), 100);
    }

    #[test]
    fn a_crop_matches_the_same_region_of_the_full_frame() {
        let mut packed = blank_frame();
        for (index, byte) in packed.iter_mut().enumerate() {
            *byte = index as u8;
        }

        let window = PartialWindow {
            x: 400,
            y: 500,
            width: 64,
            height: 4,
        };
        let cropped = crop_packed(&packed, window);

        assert_eq!(cropped.len(), window.width as usize / 2 * 4);
        for row in 0..window.height as usize {
            let source = (500 + row) * PACKED_ROW_BYTES + 200;
            let target = row * 32;
            assert_eq!(&cropped[target..target + 32], &packed[source..source + 32]);
        }
    }
}
