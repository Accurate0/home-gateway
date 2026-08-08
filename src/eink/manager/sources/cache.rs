use crate::eink::image::{center_crop_cover, content_type_for};
use crate::eink::manager::resolve::ResolvedDisplay;
use crate::error::AppError;
use crate::integrations::s3::S3;
use std::collections::HashMap;

pub const CACHE_PREFIX: &str = "eink-display/cache/";

pub async fn cache_processed_image(
    s3: &S3,
    display: &ResolvedDisplay,
    source: Vec<u8>,
    hash: &str,
    source_label: &str,
) -> Result<String, AppError> {
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

pub async fn ensure_source_hash(
    s3: &S3,
    source_key: &str,
    source: &[u8],
) -> Result<String, AppError> {
    if let Some(hash) = s3
        .get_object_metadata(source_key)
        .await?
        .and_then(|mut metadata| metadata.remove("hash"))
    {
        return Ok(hash);
    }

    let hash = crate::eink::image::content_hash(source);

    let mut metadata = HashMap::new();
    metadata.insert("hash".to_owned(), hash.clone());

    s3.put_object_with_metadata(source_key, source, content_type_for(source_key), &metadata)
        .await?;
    tracing::info!("backfilled hash metadata on {source_key}");

    Ok(hash)
}
