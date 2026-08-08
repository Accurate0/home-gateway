use crate::eink::manager::frame::{FrameContext, FrameStep};
use crate::eink::panel::{PANEL_HEIGHT, PANEL_WIDTH};
use image::RgbImage;

pub struct RotateToPortrait;

impl FrameStep for RotateToPortrait {
    fn name(&self) -> &'static str {
        "rotate"
    }

    fn fingerprint(&self, _ctx: &FrameContext) -> Option<String> {
        Some(format!("{PANEL_WIDTH}x{PANEL_HEIGHT}"))
    }

    fn apply(&self, _ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<()> {
        match img.dimensions() {
            (PANEL_HEIGHT, PANEL_WIDTH) => {
                *img = image::imageops::rotate90(img);
                Ok(())
            }
            (PANEL_WIDTH, PANEL_HEIGHT) => Ok(()),
            (w, h) => Err(anyhow::anyhow!(
                "image dimensions must be {PANEL_HEIGHT}x{PANEL_WIDTH} (will be rotated) or \
                 {PANEL_WIDTH}x{PANEL_HEIGHT}, got {w}x{h}"
            )),
        }
    }
}
