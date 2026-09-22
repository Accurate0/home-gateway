pub mod frame;
pub mod plan;
pub mod resolve;
pub mod source;
pub mod sources;
pub mod steps;

mod config;
mod store;

use crate::device_registry::DeviceRegistry;
use crate::eink::flag::epd_flag_config;
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
    eink: crate::repo::EinkRepo,
    s3: S3,
    feature_flag_client: FeatureFlagClient,
    devices: DeviceRegistry,
    settings: SettingsContainer,
    reddit: Reddit,
    sources: Arc<SourceRegistry>,
    sleep: Arc<SleepSource>,
    frames: Arc<FramePipeline>,
    packed_frames: moka::future::Cache<String, bytes::Bytes>,
}

const PACKED_CACHE_BYTES: u64 = 16 * 1024 * 1024;
const PACKED_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(6 * 60 * 60);

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
            eink: crate::repo::EinkRepo::new(db.clone()),
            db,
            s3,
            feature_flag_client,
            devices,
            settings,
            reddit,
            sources: Arc::new(sources),
            sleep: Arc::new(SleepSource),
            frames: Arc::new(frames),
            packed_frames: moka::future::Cache::builder()
                .max_capacity(PACKED_CACHE_BYTES)
                .weigher(|_, frame: &bytes::Bytes| frame.len().try_into().unwrap_or(u32::MAX))
                .time_to_live(PACKED_CACHE_TTL)
                .build(),
        }
    }

    pub fn device_ids(&self) -> Vec<String> {
        self.devices
            .eink_displays()
            .map(|(address, _)| address.clone())
            .collect()
    }

    pub async fn resolve(&self, device_id: &str) -> Option<ResolvedDisplay> {
        let display = self.devices.eink_display(device_id)?.clone();

        let flag = epd_flag_config(&self.feature_flag_client, &self.devices, device_id).await;

        Some(ResolvedDisplay::resolve(
            device_id,
            &display,
            &self.settings.eink_display,
            &flag,
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
            eink: &self.eink,
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

        self.warm_packed(display).await;

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

            self.warm_packed(display).await;
        }

        Ok(prepared)
    }

    #[tracing::instrument(name = "eink.plan", skip_all, fields(device_id = tracing::field::Empty))]
    pub async fn plan(&self, display: &ResolvedDisplay) -> Option<RenderPlan> {
        tracing::Span::current().record("device_id", display.device_id.as_str());

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

    #[tracing::instrument(
        name = "eink.ensure_packed",
        skip_all,
        fields(hash = %plan.hash, cached = tracing::field::Empty)
    )]
    pub async fn ensure_packed(&self, plan: &RenderPlan) -> Option<bytes::Bytes> {
        if let Some(frame) = self.packed_frame(&plan.hash).await {
            tracing::Span::current().record("cached", true);
            return Some(frame);
        }

        tracing::Span::current().record("cached", false);

        match self.render_packed(plan).await {
            Ok(packed) => Some(self.store_packed(&plan.hash, packed).await),
            Err(e) => {
                tracing::warn!(
                    hash = %plan.hash,
                    "failed to prepare the packed frame: {}",
                    e.message()
                );
                None
            }
        }
    }

    #[tracing::instrument(
        name = "eink.warm_packed",
        skip_all,
        fields(device_id = %resolved.device_id)
    )]
    pub async fn warm_packed(&self, resolved: &ResolvedDisplay) {
        let Some(plan) = self.plan(resolved).await else {
            return;
        };

        match self.ensure_packed(&plan).await {
            Some(_) => {
                tracing::info!(hash = %plan.hash, "warmed the packed frame ahead of the wake")
            }
            None => tracing::warn!(hash = %plan.hash, "failed to warm the packed frame"),
        }
    }

    #[tracing::instrument(name = "eink.prefill_packed", skip_all)]
    pub async fn prefill_packed(&self) {
        let warmed = futures::future::join_all(self.device_ids().into_iter().map(|device_id| {
            let manager = self.clone();

            async move {
                let Some(display) = manager.resolve(&device_id).await else {
                    return false;
                };

                let Some(plan) = manager.plan(&display).await else {
                    return false;
                };

                manager.packed_frame(&plan.hash).await.is_some()
            }
        }))
        .await;

        tracing::info!(
            warmed = warmed.iter().filter(|hit| **hit).count(),
            displays = warmed.len(),
            "prefilled the packed frame cache"
        );
    }

    pub async fn packed_frame(&self, hash: &str) -> Option<bytes::Bytes> {
        if let Some(frame) = self.packed_frames.get(hash).await {
            return Some(frame);
        }

        let key = match packed_cache_key(hash) {
            Some(key) => key,
            None => {
                tracing::error!(hash = %hash, "refusing to fetch a frame whose hash is malformed");
                return None;
            }
        };

        match self.s3.find_object(&key).await {
            Ok(Some(bytes)) => {
                let frame = bytes::Bytes::from(bytes);
                self.packed_frames
                    .insert(hash.to_string(), frame.clone())
                    .await;

                Some(frame)
            }
            Ok(None) => {
                tracing::debug!(key = %key, "packed frame is not in the s3 cache");
                None
            }
            Err(e) => {
                tracing::warn!(key = %key, "failed to read the packed frame cache: {e}");
                None
            }
        }
    }

    pub async fn store_packed(&self, hash: &str, packed: Vec<u8>) -> bytes::Bytes {
        let frame = bytes::Bytes::from(packed);

        self.packed_frames
            .insert(hash.to_string(), frame.clone())
            .await;

        if let Some(key) = packed_cache_key(hash) {
            let s3 = self.s3.clone();
            let upload = frame.clone();

            tokio::spawn(async move {
                if let Err(e) = s3.put_object(&key, &upload, None).await {
                    tracing::warn!(key = %key, "failed to cache packed frame: {e}");
                }
            });
        }

        frame
    }

    async fn render_packed(&self, plan: &RenderPlan) -> Result<Vec<u8>, AppError> {
        let source = self.s3.get_object(&plan.image_key).await?;

        let frames = self.frames.clone();
        let frame = FrameContext {
            crop_to: plan.frame.crop_to,
            sleep_label: plan.frame.sleep_label.clone(),
        };

        let span = tracing::info_span!("eink.frame.render", image_key = %plan.image_key);
        let packed =
            tokio::task::spawn_blocking(move || span.in_scope(|| frames.run(&source, &frame)))
                .await
                .map_err(|e| anyhow::anyhow!("join error: {e}"))??;

        Ok(packed)
    }
}
