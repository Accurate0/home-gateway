use crate::eink::manager::frame::{FrameContext, FrameStep};
use ab_glyph::{FontRef, PxScale};
use image::RgbImage;
use imageproc::drawing::{draw_text_mut, text_size};

const LABEL_FONT: &[u8] = include_bytes!("../../../../assets/LiberationSans-Bold.ttf");
const LABEL_SCALE: f32 = 120.0;
const LABEL_MARGIN: i32 = 48;

pub struct SleepLabel;

impl FrameStep for SleepLabel {
    fn name(&self) -> &'static str {
        "sleep_label"
    }

    fn fingerprint(&self, ctx: &FrameContext) -> Option<String> {
        ctx.sleep_label.clone()
    }

    fn apply(&self, ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<()> {
        let Some(label) = &ctx.sleep_label else {
            return Ok(());
        };

        let Ok(font) = FontRef::try_from_slice(LABEL_FONT) else {
            tracing::warn!("failed to load label font, skipping sleep label");
            return Ok(());
        };

        let scale = PxScale::from(LABEL_SCALE);
        let (text_w, text_h) = text_size(scale, &font, label);

        let (img_w, img_h) = img.dimensions();
        let x = img_w as i32 - text_w as i32 - LABEL_MARGIN;
        let y = img_h as i32 - text_h as i32 - LABEL_MARGIN;

        draw_text_mut(img, image::Rgb([0, 0, 0]), x, y, scale, &font, label);

        Ok(())
    }
}
