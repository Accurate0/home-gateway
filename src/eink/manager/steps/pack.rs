use crate::eink::manager::frame::{FrameContext, FrameEncoder};
use image::RgbImage;
use image::imageops::ColorMap;

pub struct FloydSteinbergPacked;

impl FrameEncoder for FloydSteinbergPacked {
    fn name(&self) -> &'static str {
        "pack"
    }

    fn fingerprint(&self, ctx: &FrameContext) -> String {
        ctx.palette
            .iter()
            .map(|(r, g, b, index)| format!("{r}:{g}:{b}:{index}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn encode(&self, ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<Vec<u8>> {
        let indices = dither_to_palette(img, &ctx.palette);
        let (width, height) = img.dimensions();

        let mut packed = Vec::with_capacity((width * height / 2) as usize);
        for y in 0..height {
            for x in (0..width).step_by(2) {
                let left = indices[(y * width + x) as usize];
                let right = indices[(y * width + x + 1) as usize];
                packed.push((left << 4) | right);
            }
        }

        Ok(packed)
    }
}

struct PanelPalette {
    colors: Vec<(f32, f32, f32, u8)>,
}

impl ColorMap for PanelPalette {
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

fn dither_to_palette(img: &mut RgbImage, palette: &[(f32, f32, f32, u8)]) -> Vec<u8> {
    let map = PanelPalette {
        colors: palette.to_vec(),
    };

    image::imageops::dither(img, &map);

    img.pixels()
        .map(|pixel| map.colors[map.index_of(pixel)].3)
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

        let mut img = RgbImage::from_pixel(8, 8, image::Rgb([250, 10, 8]));
        let indices = dither_to_palette(&mut img, &palette);

        assert_eq!(indices.len(), 64);
        assert!(indices.iter().all(|&index| index == 3));
        assert!(img.pixels().all(|pixel| pixel.0 == [255, 0, 0]));
    }

    #[test]
    fn dither_only_emits_palette_indices() {
        let palette = vec![(0.0, 0.0, 0.0, 0u8), (255.0, 255.0, 255.0, 1u8)];

        let mut img = RgbImage::from_pixel(16, 16, image::Rgb([128, 128, 128]));
        let indices = dither_to_palette(&mut img, &palette);

        assert!(indices.iter().all(|index| [0, 1].contains(index)));
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }
}
