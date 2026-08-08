use crate::eink::image::center_crop_cover;
use crate::eink::manager::frame::{FrameContext, FrameStep};
use image::RgbImage;

pub struct CropCover;

impl FrameStep for CropCover {
    fn name(&self) -> &'static str {
        "crop"
    }

    fn fingerprint(&self, ctx: &FrameContext) -> Option<String> {
        ctx.crop_to.map(|(w, h)| format!("{w}x{h}"))
    }

    fn apply(&self, ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<()> {
        let Some((target_w, target_h)) = ctx.crop_to else {
            return Ok(());
        };

        if img.dimensions() == (target_w, target_h) {
            return Ok(());
        }

        *img = center_crop_cover(img, target_w, target_h);

        Ok(())
    }
}
