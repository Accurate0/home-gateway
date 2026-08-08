pub mod frame;
pub mod plan;
pub mod resolve;
pub mod source;
pub mod sources;
pub mod steps;

mod config;
mod store;

use crate::device_registry::DeviceRegistry;
use crate::eink::flag::{epd_flag_config, epd_palette};
use crate::eink::panel::packed_cache_key;
use crate::error::AppError;
use crate::integrations::feature_flag::FeatureFlagClient;
use crate::integrations::reddit::Reddit;
use crate::integrations::s3::S3;
use crate::settings::SettingsContainer;
use frame::{FrameContext, FramePipeline};
use plan::{RenderPlan, render_hash};
use resolve::ResolvedDisplay;
use source::{
    ImageSource, Prepared, ScreenshotBackend, SourceContext, SourceImage, SourceRegistry,
};
use sources::{AlbumSource, DashboardSource, RedditSource, SleepSource};
use std::sync::Arc;
use steps::{CropCover, FloydSteinbergPacked, RotateToPortrait, SleepLabel};

#[derive(Clone)]
pub struct EinkDisplayManager {
    db: sqlx::Pool<sqlx::Postgres>,
    s3: S3,
    feature_flag_client: FeatureFlagClient,
    devices: DeviceRegistry,
    settings: SettingsContainer,
    reddit: Reddit,
    sources: Arc<SourceRegistry>,
    sleep: Arc<SleepSource>,
    frames: Arc<FramePipeline>,
}

impl EinkDisplayManager {
    pub fn new(
        db: sqlx::Pool<sqlx::Postgres>,
        s3: S3,
        feature_flag_client: FeatureFlagClient,
        devices: DeviceRegistry,
        settings: SettingsContainer,
        reddit: Reddit,
    ) -> Self {
        let sources = SourceRegistry::new()
            .register(DashboardSource::default())
            .register(AlbumSource)
            .register(RedditSource);

        let frames = FramePipeline::new(FloydSteinbergPacked)
            .register(CropCover)
            .register(SleepLabel)
            .register(RotateToPortrait);

        Self {
            db,
            s3,
            feature_flag_client,
            devices,
            settings,
            reddit,
            sources: Arc::new(sources),
            sleep: Arc::new(SleepSource),
            frames: Arc::new(frames),
        }
    }

    pub fn device_ids(&self) -> Vec<String> {
        self.devices.eink_displays().keys().cloned().collect()
    }

    pub async fn resolve(&self, device_id: &str) -> Option<ResolvedDisplay> {
        let display = self.devices.eink_display(device_id)?.clone();

        let flag = epd_flag_config(&self.feature_flag_client, &self.devices, device_id).await;

        let palette = epd_palette(
            &self.feature_flag_client,
            &self.devices,
            &self.settings.eink_display.palette,
            device_id,
        )
        .await;

        Some(ResolvedDisplay::resolve(
            device_id,
            &display,
            &self.settings.eink_display,
            &flag,
            palette,
        ))
    }

    fn source(&self, resolved: &ResolvedDisplay) -> Option<&dyn ImageSource> {
        self.sources.select(resolved.mode).or_else(|| {
            tracing::warn!(
                device_id = resolved.device_id.as_str(),
                "no image source registered for mode {:?}",
                resolved.mode
            );
            None
        })
    }

    fn context<'a>(
        &'a self,
        display: &'a ResolvedDisplay,
        screenshots: &'a dyn ScreenshotBackend,
    ) -> SourceContext<'a> {
        SourceContext {
            display,
            db: &self.db,
            s3: &self.s3,
            reddit: &self.reddit,
            screenshots,
        }
    }

    pub async fn refresh_source(
        &self,
        display: &ResolvedDisplay,
        screenshots: &dyn ScreenshotBackend,
    ) -> Result<Option<SourceImage>, AppError> {
        let Some(source) = self.source(display) else {
            return Ok(None);
        };

        let Some(image) = source.acquire(&self.context(display, screenshots)).await? else {
            return Ok(None);
        };

        self.store_render(&display.device_id, &display.name, &image)
            .await?;

        Ok(Some(image))
    }

    pub async fn prepare_source(
        &self,
        display: &ResolvedDisplay,
        screenshots: &dyn ScreenshotBackend,
    ) -> Result<Prepared, AppError> {
        if display.sleep.is_some() {
            let device_id = display.device_id.as_str();
            tracing::info!(
                device_id,
                "sleeping, skipping the source render the sleep image would discard"
            );

            return Ok(Prepared::Skip);
        }

        let Some(source) = self.source(display) else {
            return Ok(Prepared::Skip);
        };

        let prepared = source.prepare(&self.context(display, screenshots)).await?;

        if let Prepared::Ready(image) = &prepared {
            self.store_render(&display.device_id, &display.name, image)
                .await?;
        }

        Ok(prepared)
    }

    pub async fn plan(&self, display: &ResolvedDisplay) -> Option<RenderPlan> {
        let sleep_image = match display.sleep {
            Some(sleep) => self.sleep.acquire(&self.s3, display, sleep).await,
            None => None,
        };

        let (image, frame) = match sleep_image {
            Some(image_key) => (
                SourceImage {
                    content_hash: image_key.clone(),
                    image_key,
                },
                FrameContext {
                    crop_to: Some(display.target_dims()),
                    sleep_label: display.sleep_label(),
                    palette: display.palette.clone(),
                },
            ),
            None => {
                let device_id = display.device_id.as_str();

                let Some(image) = self.stored_render(device_id).await else {
                    tracing::warn!(device_id, "no rendered image stored yet");
                    return None;
                };

                (
                    image,
                    FrameContext {
                        crop_to: None,
                        sleep_label: None,
                        palette: display.palette.clone(),
                    },
                )
            }
        };

        let mut parts = vec![
            image.image_key.clone(),
            image.content_hash.clone(),
            display.clear_screen.to_string(),
        ];
        parts.extend(self.frames.fingerprint(&frame));

        let hash = render_hash(&parts.iter().map(String::as_str).collect::<Vec<_>>());

        Some(RenderPlan {
            image_key: image.image_key,
            sleep: display.sleep,
            hash,
            frame,
        })
    }

    pub async fn ensure_packed(&self, plan: &RenderPlan) -> bool {
        let key = packed_cache_key(plan.hash.clone());

        match self.s3.get_object_metadata(&key).await {
            Ok(Some(_)) => return true,
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(key = %key, "failed to check the packed cache: {e}");
                return false;
            }
        }

        match self.render_packed(plan).await {
            Ok(packed) => {
                if let Err(e) = self.s3.put_object(&key, &packed, None).await {
                    tracing::warn!(key = %key, "failed to cache packed frame: {e}");
                }
                true
            }
            Err(e) => {
                tracing::warn!(key = %key, "failed to prepare the packed frame: {}", e.message());
                false
            }
        }
    }

    async fn render_packed(&self, plan: &RenderPlan) -> Result<Vec<u8>, AppError> {
        let source = self.s3.get_object(&plan.image_key).await?;

        let frames = self.frames.clone();
        let frame = FrameContext {
            crop_to: plan.frame.crop_to,
            sleep_label: plan.frame.sleep_label.clone(),
            palette: plan.frame.palette.clone(),
        };

        let packed = tokio::task::spawn_blocking(move || frames.run(&source, &frame))
            .await
            .map_err(|e| anyhow::anyhow!("join error: {e}"))??;

        Ok(packed)
    }
}
