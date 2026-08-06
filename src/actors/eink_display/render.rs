use super::{CACHE_PREFIX, EInkActorState, EInkDisplayActor};
use crate::settings::EinkDisplaySettings;
use crate::reddit::image_url;
use crate::s3::OptionalObjectResponse;
use crate::settings::{Album, RedditFeed};
use chromiumoxide::{
    cdp::browser_protocol::{
        emulation::{SetDeviceMetricsOverrideParams, SetLocaleOverrideParams},
        page::CaptureScreenshotFormat,
    },
    page::ScreenshotParams,
};
use rand::seq::IndexedRandom;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

impl EInkDisplayActor {
    pub(super) async fn save_index_if_new(
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

    pub(super) async fn render_web(
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

    pub(super) async fn render_album(
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

    pub(super) async fn cache_processed_image(
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

    pub(super) async fn render_reddit(
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

    pub(super) fn dashboard_key(view_name: &str, orientation: &str) -> String {
        format!("eink-display/dashboard/{view_name}-{orientation}.png")
    }
}

pub(super) fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub(super) fn content_type_for(key: &str) -> Option<&'static str> {
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
