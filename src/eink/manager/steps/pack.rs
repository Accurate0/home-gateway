use crate::eink::manager::frame::{FrameContext, FrameEncoder};
use crate::settings::devices::eink::PALETTE_COLORS;
use image::RgbImage;
use image::imageops::ColorMap;
use std::sync::OnceLock;

pub struct FloydSteinbergPacked;

#[derive(Clone, Copy)]
pub struct PaletteColor {
    r: f32,
    g: f32,
    b: f32,
    index: u8,
}

impl PaletteColor {
    fn rgb(&self) -> [u8; 3] {
        [self.r as u8, self.g as u8, self.b as u8]
    }
}

impl FrameEncoder for FloydSteinbergPacked {
    fn name(&self) -> &'static str {
        "pack"
    }

    fn fingerprint(&self, _ctx: &FrameContext) -> String {
        panel_palette()
            .iter()
            .map(|color| format!("{}:{}:{}:{}", color.r, color.g, color.b, color.index))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn encode(&self, _ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<Vec<u8>> {
        let indices = dither_to_palette(img, panel_palette());
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

fn panel_palette() -> &'static [PaletteColor] {
    static PALETTE: OnceLock<Vec<PaletteColor>> = OnceLock::new();

    PALETTE.get_or_init(|| {
        PALETTE_COLORS
            .iter()
            .map(|&(_, r, g, b, index)| PaletteColor { r, g, b, index })
            .collect()
    })
}

struct PanelPalette<'a> {
    colors: &'a [PaletteColor],
}

impl PanelPalette<'_> {
    fn exact_index_of(&self, color: &image::Rgb<u8>) -> Option<usize> {
        self.colors
            .iter()
            .position(|candidate| candidate.rgb() == color.0)
    }
}

impl ColorMap for PanelPalette<'_> {
    type Color = image::Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let [r, g, b] = color.0;
        let (r, g, b) = (r as f32, g as f32, b as f32);

        let mut closest = 0;
        let mut min_dist = f32::MAX;

        for (i, candidate) in self.colors.iter().enumerate() {
            let dist =
                (r - candidate.r).powi(2) + (g - candidate.g).powi(2) + (b - candidate.b).powi(2);
            if dist < min_dist {
                min_dist = dist;
                closest = i;
            }
        }

        closest
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        self.colors.get(index).map(|color| image::Rgb(color.rgb()))
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

fn dither_to_palette(img: &mut RgbImage, palette: &[PaletteColor]) -> Vec<u8> {
    let map = PanelPalette { colors: palette };

    image::imageops::dither(img, &map);

    img.pixels()
        .map(|pixel| {
            let index = map
                .exact_index_of(pixel)
                .unwrap_or_else(|| map.index_of(pixel));

            map.colors[index].index
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(colors: &[(f32, f32, f32, u8)]) -> Vec<PaletteColor> {
        colors
            .iter()
            .map(|&(r, g, b, index)| PaletteColor { r, g, b, index })
            .collect()
    }

    #[test]
    fn dither_maps_flat_colours_to_their_palette_index() {
        let palette = palette(&[
            (0.0, 0.0, 0.0, 0),
            (255.0, 255.0, 255.0, 1),
            (255.0, 0.0, 0.0, 3),
        ]);

        let mut img = RgbImage::from_pixel(8, 8, image::Rgb([250, 10, 8]));
        let indices = dither_to_palette(&mut img, &palette);

        assert_eq!(indices.len(), 64);
        assert!(indices.iter().all(|&index| index == 3));
        assert!(img.pixels().all(|pixel| pixel.0 == [255, 0, 0]));
    }

    #[test]
    fn dither_only_emits_palette_indices() {
        let palette = palette(&[(0.0, 0.0, 0.0, 0), (255.0, 255.0, 255.0, 1)]);

        let mut img = RgbImage::from_pixel(16, 16, image::Rgb([128, 128, 128]));
        let indices = dither_to_palette(&mut img, &palette);

        assert!(indices.iter().all(|index| [0, 1].contains(index)));
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn exact_lookup_matches_the_nearest_colour_search() {
        let colors = panel_palette();
        let map = PanelPalette { colors };

        for color in colors {
            let pixel = image::Rgb(color.rgb());
            assert_eq!(map.exact_index_of(&pixel), Some(map.index_of(&pixel)));
        }
    }
}
