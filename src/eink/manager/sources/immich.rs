use super::cache::cache_processed_image;
use crate::eink::image::content_hash;
use crate::eink::manager::source::{ImageSource, SourceContext, SourceImage};
use crate::error::AppError;
use crate::integrations::immich::types::ImmichAsset;
use crate::settings::{EinkMode, Orientation};
use rand::seq::IndexedRandom;

const DECODABLE_TYPES: [&str; 5] = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/heic",
    "image/heif",
];

pub struct ImmichSource;

#[async_trait::async_trait]
impl ImageSource for ImmichSource {
    fn name(&self) -> &'static str {
        "immich"
    }

    fn mode(&self) -> EinkMode {
        EinkMode::Immich
    }

    async fn acquire(&self, ctx: &SourceContext<'_>) -> Result<Option<SourceImage>, AppError> {
        let Some(immich) = ctx.immich else {
            tracing::warn!(
                "immich mode selected for `{}` but the immich integration is disabled, skipping render",
                ctx.display.name
            );
            return Ok(None);
        };

        let Some(album) = &ctx.display.immich_album else {
            tracing::warn!(
                "immich mode selected for `{}` but no album is configured, skipping render",
                ctx.display.name
            );
            return Ok(None);
        };

        let album_id = immich.album_id(album).await?;
        let assets = immich.random_assets(&album_id, immich.candidates()).await?;

        let Some(asset) = pick(&assets, ctx.display.orientation) else {
            tracing::warn!("immich album `{album}` has no decodable images, skipping render");
            return Ok(None);
        };

        tracing::info!("selected immich asset {} from album `{album}`", asset.id);

        let source = immich.download_original(&asset.id).await?;
        let content_hash = content_hash(&source);
        let label = format!("immich:{}", asset.id);

        let cached =
            cache_processed_image(ctx.s3, ctx.display, source, &content_hash, &label).await?;

        Ok(Some(SourceImage {
            image_key: cached.key,
            content_hash,
            payload: Some(cached.payload),
        }))
    }
}

fn pick(assets: &[ImmichAsset], orientation: Orientation) -> Option<&ImmichAsset> {
    let decodable = assets
        .iter()
        .filter(|asset| {
            asset
                .original_mime_type
                .as_deref()
                .is_some_and(|mime| DECODABLE_TYPES.contains(&mime))
        })
        .collect::<Vec<_>>();

    let matching = decodable
        .iter()
        .copied()
        .filter(|asset| matches_orientation(asset, orientation))
        .collect::<Vec<_>>();

    let mut rng = rand::rng();

    matching
        .choose(&mut rng)
        .or_else(|| decodable.choose(&mut rng))
        .copied()
}

fn matches_orientation(asset: &ImmichAsset, orientation: Orientation) -> bool {
    let Some((width, height)) = asset.displayed_dims() else {
        return false;
    };

    match orientation {
        Orientation::Portrait => height >= width,
        Orientation::Landscape => width >= height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::immich::types::ExifInfo;
    use pretty_assertions::assert_eq;

    fn asset(id: &str, mime: &str, dims: Option<(f64, f64)>, orientation: &str) -> ImmichAsset {
        ImmichAsset {
            id: id.to_owned(),
            original_mime_type: Some(mime.to_owned()),
            exif_info: dims.map(|(width, height)| ExifInfo {
                exif_image_width: Some(width),
                exif_image_height: Some(height),
                orientation: Some(orientation.to_owned()),
            }),
        }
    }

    #[test]
    fn a_matching_orientation_is_preferred() {
        let assets = [
            asset("wide", "image/jpeg", Some((4000.0, 3000.0)), "1"),
            asset("tall", "image/heic", Some((4032.0, 3024.0)), "6"),
        ];

        for _ in 0..20 {
            assert_eq!(
                pick(&assets, Orientation::Portrait).map(|a| a.id.as_str()),
                Some("tall")
            );
            assert_eq!(
                pick(&assets, Orientation::Landscape).map(|a| a.id.as_str()),
                Some("wide")
            );
        }
    }

    #[test]
    fn any_decodable_asset_is_used_when_none_match() {
        let assets = [
            asset("wide", "image/jpeg", Some((4000.0, 3000.0)), "1"),
            asset("unknown", "image/png", None, "1"),
        ];

        assert!(pick(&assets, Orientation::Portrait).is_some());
    }

    #[test]
    fn undecodable_types_are_never_picked() {
        let assets = [
            asset("raw", "image/x-adobe-dng", Some((3000.0, 4000.0)), "1"),
            ImmichAsset {
                id: "untyped".to_owned(),
                original_mime_type: None,
                exif_info: None,
            },
        ];

        assert!(pick(&assets, Orientation::Portrait).is_none());
    }
}
