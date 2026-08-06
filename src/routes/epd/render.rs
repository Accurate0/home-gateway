use crate::error::AppError;
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{draw_text_mut, text_size};

const LABEL_FONT: &[u8] = include_bytes!("../../../assets/LiberationSans-Bold.ttf");

pub(super) async fn render_and_cache(
    s3: &crate::s3::S3,
    image_key: &str,
    sleep_label: Option<String>,
    palette: &[(f32, f32, f32, u8)],
    cache_key: &str,
) -> Result<Vec<u8>, AppError> {
    let image_response = s3.get_object(image_key).await?;
    let packed = render_packed(image_response, sleep_label, palette.to_vec()).await?;

    if let Err(e) = s3.put_object(cache_key, &packed, None).await {
        tracing::warn!(key = %cache_key, "failed to cache packed frame: {e}");
    }

    Ok(packed)
}

async fn render_packed(
    image_response: Vec<u8>,
    sleep_label: Option<String>,
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

        let indices = dither_to_palette(&mut img, &palette);

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

struct PanelPalette {
    colors: Vec<(f32, f32, f32, u8)>,
}

impl image::imageops::ColorMap for PanelPalette {
    type Color = image::Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let [r, g, b] = color.0;
        let (r, g, b) = (r as f32, g as f32, b as f32);

        let mut closest = 0;
        let mut min_dist = f32::MAX;

        for (i, &(pr, pg, pb, _)) in self.colors.iter().enumerate() {
            let dist = (r - pr).powi(2) + (g - pg).powi(2) + (b - pb).powi(2);
            if dist < min_dist {
                min_dist = dist;
                closest = i;
            }
        }

        closest
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        self.colors
            .get(index)
            .map(|&(r, g, b, _)| image::Rgb([r as u8, g as u8, b as u8]))
    }

    fn has_lookup(&self) -> bool {
        true
    }

    fn map_color(&self, color: &mut Self::Color) {
        let index = self.index_of(color);
        if let Some(mapped) = self.lookup(index) {
            *color = mapped;
        }
    }
}

fn dither_to_palette(img: &mut image::RgbImage, palette: &[(f32, f32, f32, u8)]) -> Vec<u8> {
    let map = PanelPalette {
        colors: palette.to_vec(),
    };

    image::imageops::dither(img, &map);

    img.pixels()
        .map(|pixel| {
            let (_, _, _, index) = map.colors[image::imageops::ColorMap::index_of(&map, pixel)];
            index
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dither_maps_flat_colours_to_their_palette_index() {
        let palette = vec![
            (0.0, 0.0, 0.0, 0u8),
            (255.0, 255.0, 255.0, 1u8),
            (255.0, 0.0, 0.0, 3u8),
        ];

        let mut img = image::RgbImage::from_pixel(8, 8, image::Rgb([250, 10, 8]));
        let indices = dither_to_palette(&mut img, &palette);

        assert_eq!(indices.len(), 64);
        assert!(indices.iter().all(|&index| index == 3));
        assert!(img.pixels().all(|pixel| pixel.0 == [255, 0, 0]));
    }

    #[test]
    fn dither_only_emits_palette_indices() {
        let palette = vec![(0.0, 0.0, 0.0, 0u8), (255.0, 255.0, 255.0, 1u8)];

        let mut img = image::RgbImage::from_pixel(16, 16, image::Rgb([128, 128, 128]));
        let indices = dither_to_palette(&mut img, &palette);

        assert!(indices.iter().all(|index| [0, 1].contains(index)));
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }
}
