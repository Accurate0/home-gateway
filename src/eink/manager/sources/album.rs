use super::cache::{CACHE_PREFIX, cache_processed_image, ensure_source_hash};
use crate::eink::manager::source::{ImageSource, SourceContext, SourceImage};
use crate::error::AppError;
use crate::settings::EinkMode;
use rand::seq::IndexedRandom;

pub struct AlbumSource;

#[async_trait::async_trait]
impl ImageSource for AlbumSource {
    fn name(&self) -> &'static str {
        "album"
    }

    fn mode(&self) -> EinkMode {
        EinkMode::Album
    }

    async fn acquire(&self, ctx: &SourceContext<'_>) -> Result<Option<SourceImage>, AppError> {
        let album = &ctx.display.album;

        let images: Vec<String> = ctx
            .s3
            .list_objects(&album.prefix)
            .await?
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

        let source = ctx.s3.get_object(&source_key).await?;
        let content_hash = ensure_source_hash(ctx.s3, &source_key, &source).await?;

        let image_key =
            cache_processed_image(ctx.s3, ctx.display, source, &content_hash, &source_key).await?;

        Ok(Some(SourceImage {
            image_key,
            content_hash,
        }))
    }
}
