use crate::battery::{BatteryChemistry, voltage_to_percentage};
use crate::device_registry::EinkDisplaySettings;
use crate::reddit::{Reddit, image_url};
use crate::s3::OptionalObjectResponse;
use crate::settings::{Album, EinkMode, RedditFeed};
use crate::types::SharedActorState;
use chromiumoxide::{
    Browser, BrowserConfig,
    cdp::browser_protocol::{
        emulation::{SetDeviceMetricsOverrideParams, SetLocaleOverrideParams},
        page::CaptureScreenshotFormat,
    },
    handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use ractor::{Actor, RpcReplyPort};
use rand::seq::IndexedRandom;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;

const CACHE_PREFIX: &str = "eink-display/cache/";

pub mod types;

pub enum EInkDisplayMessage {
    TakeScreenshot {
        device_id: Option<String>,
    },
    PrepareRender {
        device_id: String,
        reply: RpcReplyPort<()>,
    },
    BatteryReport {
        device_id: String,
        battery_voltage: f64,
        is_charging: Option<bool>,
        battery_chemistry: Option<BatteryChemistry>,
        battery_kind: Option<String>,
    },
    ConfigRequest {
        device_id: String,
    },
}

pub struct EInkDisplayActor {
    pub shared_actor_state: SharedActorState,
    pub reddit: Reddit,
}

impl EInkDisplayActor {
    pub const NAME: &str = "eink_display";
    #[cfg(not(debug_assertions))]
    pub const INDEX_PATH: &str = "file:///tmp/index.html";
    pub const INDEX_FS_PATH: &str = "/tmp/index.html";
    #[cfg(debug_assertions)]
    pub const INDEX_PATH: &str =
        "file:///home/anurag/Projects/home-gateway/eink-display-web/dist/index.html";
}

pub struct EInkActorState {
    index_html_etag: Option<String>,
    // Optional so the gateway still starts where Chromium can't launch (e.g. a
    // dev box or sandbox with no usable /dev/shm or dbus). Screenshotting is
    // skipped in that case rather than crashing the whole process.
    browser: Option<Browser>,
    #[allow(unused)]
    browser_handle: Option<JoinHandle<()>>,
}

impl EInkDisplayActor {
    async fn save_index_if_new(
        &self,
        state: &mut EInkActorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let etag = state.index_html_etag.as_deref();
        let optional_index = self
            .shared_actor_state
            .s3
            .get_object_optional("eink-display/web/index.html", etag)
            .await?;

        match optional_index {
            OptionalObjectResponse::ExistingObjectIsValid => {
                tracing::info!("304 from server, not replacing file");
            }
            OptionalObjectResponse::ObjectUpdated(object_response) => {
                tracing::info!("index fetched: etag {:?}", object_response.etag);
                let mut index_file = File::create(Self::INDEX_FS_PATH).await?;
                index_file.write_all(&object_response.payload).await?;
                state.index_html_etag = Some(object_response.etag);
            }
        };

        Ok(())
    }

    async fn render_web(
        &self,
        state: &mut EInkActorState,
        display: &EinkDisplaySettings,
        settle: Duration,
        query: Option<&str>,
    ) -> Result<Option<Vec<u8>>, ractor::ActorProcessingErr> {
        if !cfg!(debug_assertions) {
            self.save_index_if_new(state).await?;
        }

        let Some(browser) = &state.browser else {
            tracing::warn!("skipping screenshot: chromium not available");
            return Ok(None);
        };

        let orientation_param = format!("orientation={}", display.orientation_str());
        let query = match query {
            Some(query) if !query.is_empty() => format!("{query}&{orientation_param}"),
            _ => orientation_param,
        };
        let url = format!("{}?{query}", Self::INDEX_PATH);

        let original_page = browser.new_page(url).await?;
        tracing::info!("navigating to page");

        let (width, height) = display.target_dims();
        original_page
            .execute(
                SetDeviceMetricsOverrideParams::builder()
                    .width(width as i64)
                    .height(height as i64)
                    .device_scale_factor(1.0)
                    .mobile(false)
                    .build()?,
            )
            .await?;

        tracing::info!("setting locale and timezone");
        let page = original_page.emulate_timezone("Australia/Perth").await?;
        let page = page
            .emulate_locale(SetLocaleOverrideParams::builder().locale("en-AU").build())
            .await?;
        page.reload().await?;

        tokio::time::sleep(settle).await;

        tracing::info!("screenshot taken");
        let image = page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .full_page(false)
                    .build(),
            )
            .await?;

        original_page.close().await?;

        Ok(Some(image))
    }

    async fn render_album(
        &self,
        display: &EinkDisplaySettings,
        album: &Album,
    ) -> Result<Option<(String, String)>, ractor::ActorProcessingErr> {
        let s3 = &self.shared_actor_state.s3;

        let images = s3.list_objects(&album.prefix).await?;
        let images: Vec<String> = images
            .into_iter()
            .filter(|key| !key.starts_with(CACHE_PREFIX))
            .collect();
        let Some(source_key) = images.choose(&mut rand::rng()).cloned() else {
            tracing::warn!(
                "album `{}` at `{}` has no images, skipping render",
                album.name,
                album.prefix
            );
            return Ok(None);
        };

        let source = s3.get_object(&source_key).await?;

        let hash = match s3
            .get_object_metadata(&source_key)
            .await?
            .and_then(|mut m| m.remove("hash"))
        {
            Some(hash) => hash,
            None => {
                let hash = content_hash(&source);
                let mut metadata = HashMap::new();
                metadata.insert("hash".to_owned(), hash.clone());
                let content_type = content_type_for(&source_key);
                s3.put_object_with_metadata(&source_key, &source, content_type, &metadata)
                    .await?;
                tracing::info!("backfilled hash metadata on {source_key}");
                hash
            }
        };

        let cache_key = self
            .cache_processed_image(display, source, &hash, &source_key)
            .await?;

        Ok(Some((cache_key, hash)))
    }

    async fn cache_processed_image(
        &self,
        display: &EinkDisplaySettings,
        source: Vec<u8>,
        hash: &str,
        source_label: &str,
    ) -> Result<String, ractor::ActorProcessingErr> {
        let s3 = &self.shared_actor_state.s3;

        let (target_w, target_h) = display.target_dims();
        let cache_key = format!("{CACHE_PREFIX}{hash}-{}.png", display.orientation_str());

        if s3.get_object_metadata(&cache_key).await?.is_some() {
            tracing::info!("image cache hit for {source_label} -> {cache_key}");
            return Ok(cache_key);
        }

        let processed = tokio::task::spawn_blocking(move || {
            let img = image::load_from_memory(&source)?.to_rgb8();
            let cropped = center_crop_cover(&img, target_w, target_h);
            let mut out = Vec::new();
            image::DynamicImage::ImageRgb8(cropped)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)?;
            Ok::<_, anyhow::Error>(out)
        })
        .await
        .map_err(|e| anyhow::anyhow!("join error: {e}"))??;

        let mut metadata = HashMap::new();
        metadata.insert("source_hash".to_owned(), hash.to_owned());
        s3.put_object_with_metadata(&cache_key, &processed, Some("image/png"), &metadata)
            .await?;
        tracing::info!("image cached {source_label} -> {cache_key}");

        Ok(cache_key)
    }

    async fn render_reddit(
        &self,
        display: &EinkDisplaySettings,
        feed: &RedditFeed,
    ) -> Result<Option<(String, String)>, ractor::ActorProcessingErr> {
        let posts = self
            .reddit
            .top_posts(&feed.subreddit, feed.timespan, feed.limit)
            .await?;

        let candidates: Vec<String> = posts.iter().filter_map(image_url).collect();
        let Some(url) = candidates.choose(&mut rand::rng()).cloned() else {
            tracing::warn!(
                "no usable image posts in r/{} over {}, skipping render",
                feed.subreddit,
                feed.timespan.as_str()
            );
            return Ok(None);
        };

        tracing::info!("selected reddit image {url} from r/{}", feed.subreddit);
        let source = self.reddit.download_image(&url).await?;
        let hash = content_hash(&source);

        let cache_key = self
            .cache_processed_image(display, source, &hash, &url)
            .await?;

        Ok(Some((cache_key, hash)))
    }

    fn dashboard_key(view_name: &str, orientation: &str) -> String {
        format!("eink-display/dashboard/{view_name}-{orientation}.png")
    }

    async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image_key: &str,
        image_content_hash: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, image_key, image_content_hash) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, image_content_hash = EXCLUDED.image_content_hash",
            device_id,
            name,
            image_key,
            image_content_hash,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        tracing::info!("eink display image updated for {device_id} -> {image_key}");

        Ok(())
    }

    async fn stored_image_key(
        &self,
        device_id: &str,
    ) -> Result<Option<String>, ractor::ActorProcessingErr> {
        let row = sqlx::query!(
            "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(row.and_then(|row| row.image_key))
    }

    async fn adopt_last_dashboard(
        &self,
        device_id: &str,
        display: &EinkDisplaySettings,
        image_key: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        if self.stored_image_key(device_id).await?.as_deref() == Some(image_key) {
            return Ok(());
        }

        let image = match self.shared_actor_state.s3.get_object(image_key).await {
            Ok(image) => image,
            Err(e) => {
                tracing::warn!(
                    "no previous dashboard render at {image_key} for {device_id} ({e}), \
                     serving the current frame until the screenshot lands"
                );
                return Ok(());
            }
        };

        self.store_render(device_id, &display.name, image_key, &content_hash(&image))
            .await
    }

    async fn prepare_render(
        &self,
        myself: &ractor::ActorRef<EInkDisplayMessage>,
        device_id: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(display) = self.shared_actor_state.devices.eink_display(device_id) else {
            tracing::warn!("render requested for unregistered eink display '{device_id}'");
            return Ok(());
        };
        let display = display.clone();

        let flag = crate::routes::epd::epd_flag_config(
            &self.shared_actor_state.feature_flag_client,
            &self.shared_actor_state.devices,
            device_id,
        )
        .await;

        let global = &self.shared_actor_state.settings.eink_display;

        match flag.resolve_mode(&display) {
            EinkMode::Album => {
                let album = flag.resolve_album(global, &display);
                if let Some((image_key, hash)) = self.render_album(&display, &album).await? {
                    self.store_render(device_id, &display.name, &image_key, &hash)
                        .await?;
                }
            }
            EinkMode::Reddit => {
                if let Some(feed) = flag.resolve_reddit_feed(&display)
                    && let Some((image_key, hash)) = self.render_reddit(&display, &feed).await?
                {
                    self.store_render(device_id, &display.name, &image_key, &hash)
                        .await?;
                }
            }
            EinkMode::Dashboard => {
                let view_name = flag
                    .resolve_view(global, &display)
                    .map(|view| view.name.clone())
                    .unwrap_or_else(|| "default".to_owned());
                let image_key = Self::dashboard_key(&view_name, display.orientation_str());

                self.adopt_last_dashboard(device_id, &display, &image_key)
                    .await?;

                myself.send_message(EInkDisplayMessage::TakeScreenshot {
                    device_id: Some(device_id.to_owned()),
                })?;
            }
        }

        Ok(())
    }
}

fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn content_type_for(key: &str) -> Option<&'static str> {
    let lower = key.to_ascii_lowercase();
    if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_crop_cover_matches_target_dims() {
        let src = image::RgbImage::from_pixel(800, 400, image::Rgb([10, 20, 30]));
        let out = center_crop_cover(&src, 1200, 1600);
        assert_eq!(out.dimensions(), (1200, 1600));

        let out = center_crop_cover(&src, 1600, 1200);
        assert_eq!(out.dimensions(), (1600, 1200));
    }

    #[test]
    fn content_hash_is_stable() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
        assert_ne!(content_hash(b"hello"), content_hash(b"world"));
    }
}

fn center_crop_cover(img: &image::RgbImage, target_w: u32, target_h: u32) -> image::RgbImage {
    let (w, h) = img.dimensions();
    let scale = (target_w as f32 / w as f32).max(target_h as f32 / h as f32);
    let scaled_w = (w as f32 * scale).ceil() as u32;
    let scaled_h = (h as f32 * scale).ceil() as u32;
    let scaled = image::imageops::resize(
        img,
        scaled_w.max(target_w),
        scaled_h.max(target_h),
        image::imageops::FilterType::Lanczos3,
    );
    let (sw, sh) = scaled.dimensions();
    let x = (sw - target_w) / 2;
    let y = (sh - target_h) / 2;
    image::imageops::crop_imm(&scaled, x, y, target_w, target_h).to_image()
}

impl Actor for EInkDisplayActor {
    type Msg = EInkDisplayMessage;
    type State = EInkActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let launch = BrowserConfig::builder()
            .new_headless_mode()
            .arg("--disable-crash-reporter")
            .arg("--no-crashpad")
            .arg("--no-sandbox")
            // container has a small /dev/shm; without this Chromium crashes on startup (SIGTRAP)
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .env("XDG_CONFIG_HOME", "/tmp/chromium")
            .env("XDG_CACHE_HOME", "/tmp/chromium")
            .viewport(Some(Viewport {
                width: 1600,
                height: 1200,
                device_scale_factor: None,
                emulating_mobile: false,
                is_landscape: false,
                has_touch: false,
            }))
            .build()
            .map_err(ractor::ActorProcessingErr::from);

        let (browser, browser_handle) = match launch {
            Ok(config) => match Browser::launch(config).await {
                Ok((browser, mut handler)) => {
                    let handle = tokio::spawn(async move {
                        while let Some(h) = handler.next().await {
                            if h.is_err() {
                                break;
                            }
                        }
                    });
                    (Some(browser), Some(handle))
                }
                Err(e) => {
                    tracing::warn!("chromium failed to launch, screenshots disabled: {e}");
                    (None, None)
                }
            },
            Err(e) => {
                tracing::warn!("chromium config invalid, screenshots disabled: {e}");
                (None, None)
            }
        };

        if browser.is_some() {
            myself.send_message(EInkDisplayMessage::TakeScreenshot { device_id: None })?;
        }

        Ok(EInkActorState {
            browser,
            index_html_etag: None,
            browser_handle,
        })
    }

    async fn post_stop(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        if let Some(browser) = &mut state.browser {
            browser.close().await?;
        }
        Ok(())
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            EInkDisplayMessage::TakeScreenshot { device_id } => {
                let all = self.shared_actor_state.devices.eink_displays();
                let target_ids: Vec<String> = match &device_id {
                    Some(id) => {
                        if all.contains_key(id) {
                            vec![id.clone()]
                        } else {
                            tracing::warn!("render requested for unknown eink display '{id}'");
                            return Ok(());
                        }
                    }
                    None => all.keys().cloned().collect(),
                };
                if target_ids.is_empty() {
                    tracing::debug!("no registered eink displays, skipping render");
                    return Ok(());
                }

                for device_id in target_ids {
                    let display = match self.shared_actor_state.devices.eink_display(&device_id) {
                        Some(display) => display.clone(),
                        None => continue,
                    };

                    let flag = crate::routes::epd::epd_flag_config(
                        &self.shared_actor_state.feature_flag_client,
                        &self.shared_actor_state.devices,
                        &device_id,
                    )
                    .await;

                    let global = &self.shared_actor_state.settings.eink_display;
                    let (image_key, image_content_hash) = match flag.resolve_mode(&display) {
                        EinkMode::Dashboard => {
                            let settle = display
                                .settle
                                .and_then(|s| s.to_std().ok())
                                .unwrap_or(Duration::from_secs(10));
                            let view = flag.resolve_view(global, &display);
                            let view_name = view
                                .map(|v| v.name.clone())
                                .unwrap_or_else(|| "default".to_owned());
                            let query = view.and_then(|v| v.query.clone());
                            let image = match self
                                .render_web(state, &display, settle, query.as_deref())
                                .await?
                            {
                                Some(image) => image,
                                None => continue,
                            };
                            let key = Self::dashboard_key(&view_name, display.orientation_str());
                            self.shared_actor_state
                                .s3
                                .put_object(&key, &image, Some("image/png"))
                                .await?;

                            let hash = content_hash(&image);

                            (key, hash)
                        }
                        EinkMode::Album => match self
                            .render_album(&display, &flag.resolve_album(global, &display))
                            .await?
                        {
                            Some(rendered) => rendered,
                            None => continue,
                        },
                        EinkMode::Reddit => {
                            let Some(feed) = flag.resolve_reddit_feed(&display) else {
                                continue;
                            };
                            match self.render_reddit(&display, &feed).await? {
                                Some(rendered) => rendered,
                                None => continue,
                            }
                        }
                    };

                    self.store_render(&device_id, &display.name, &image_key, &image_content_hash)
                        .await?;
                }
            }
            EInkDisplayMessage::PrepareRender { device_id, reply } => {
                let result = self.prepare_render(&myself, &device_id).await;

                if reply.send(()).is_err() {
                    tracing::warn!("prepare render reply dropped for {device_id}");
                }

                result?;
            }
            EInkDisplayMessage::BatteryReport {
                device_id,
                battery_voltage,
                is_charging,
                battery_chemistry,
                battery_kind,
            } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "battery report from unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };
                let name = display.name.clone();
                let kind = battery_kind.unwrap_or_else(|| "eink_display_firmware".to_owned());
                let chemistry = battery_chemistry.unwrap_or(BatteryChemistry::Unknown);

                sqlx::query!(
                    "INSERT INTO eink_display (device_id, name, battery_voltage, is_charging, updated_at) VALUES ($1, $2, $3, $4, now()) \
                     ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, is_charging = EXCLUDED.is_charging, updated_at = EXCLUDED.updated_at",
                    device_id,
                    name,
                    battery_voltage,
                    is_charging,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                crate::actors::battery::BatteryActor::report(
                    device_id,
                    name,
                    kind,
                    Some(battery_voltage),
                    voltage_to_percentage(chemistry, battery_voltage),
                    Some(chemistry),
                );
            }
            EInkDisplayMessage::ConfigRequest { device_id } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "config request from unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };

                sqlx::query!(
                    "INSERT INTO eink_display (device_id, name, updated_at) VALUES ($1, $2, now()) \
                     ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at",
                    device_id,
                    display.name,
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }
        }

        Ok(())
    }
}
