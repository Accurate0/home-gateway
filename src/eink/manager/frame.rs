use image::RgbImage;

pub struct FrameContext {
    pub crop_to: Option<(u32, u32)>,
    pub sleep_label: Option<String>,
    pub palette: Vec<(f32, f32, f32, u8)>,
}

pub trait FrameStep: Send + Sync {
    fn name(&self) -> &'static str;

    fn fingerprint(&self, ctx: &FrameContext) -> Option<String>;

    fn apply(&self, ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<()>;
}

pub trait FrameEncoder: Send + Sync {
    fn name(&self) -> &'static str;

    fn fingerprint(&self, ctx: &FrameContext) -> String;

    fn encode(&self, ctx: &FrameContext, img: &mut RgbImage) -> anyhow::Result<Vec<u8>>;
}

pub struct FramePipeline {
    steps: Vec<Box<dyn FrameStep>>,
    encoder: Box<dyn FrameEncoder>,
}

impl FramePipeline {
    pub fn new(encoder: impl FrameEncoder + 'static) -> Self {
        Self {
            steps: Vec::new(),
            encoder: Box::new(encoder),
        }
    }

    pub fn register(mut self, step: impl FrameStep + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn fingerprint(&self, ctx: &FrameContext) -> Vec<String> {
        let mut parts: Vec<String> = self
            .steps
            .iter()
            .filter_map(|step| {
                step.fingerprint(ctx)
                    .map(|fingerprint| format!("{}={fingerprint}", step.name()))
            })
            .collect();

        parts.push(format!(
            "{}={}",
            self.encoder.name(),
            self.encoder.fingerprint(ctx)
        ));

        parts
    }

    pub fn run(&self, source: &[u8], ctx: &FrameContext) -> anyhow::Result<Vec<u8>> {
        let mut img = image::load_from_memory(source)?.to_rgb8();

        for step in &self.steps {
            if step.fingerprint(ctx).is_none() {
                continue;
            }

            step.apply(ctx, &mut img)
                .map_err(|e| anyhow::anyhow!("eink frame step `{}` failed: {e}", step.name()))?;
        }

        self.encoder.encode(ctx, &mut img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::steps::{
        CropCover, FloydSteinbergPacked, RotateToPortrait, SleepLabel,
    };
    use crate::eink::panel::PACKED_FRAME_SIZE;
    use crate::settings::eink::PALETTE_COLORS;

    const GOLDEN_CROPPED: &str = "dc86f400cd03697fd84b27d5cd1306185720d73d25c4ab85f79dbed746cbcf3c";
    const GOLDEN_ROTATED: &str = "68ed8c71e335465d1157d1d0a6d8bed26d38d997912a776a66e2a2cd99051918";

    fn pipeline() -> FramePipeline {
        FramePipeline::new(FloydSteinbergPacked)
            .register(CropCover)
            .register(SleepLabel)
            .register(RotateToPortrait)
    }

    fn palette() -> Vec<(f32, f32, f32, u8)> {
        PALETTE_COLORS
            .iter()
            .map(|&(_, r, g, b, index)| (r, g, b, index))
            .collect()
    }

    fn source_png() -> Vec<u8> {
        let mut img = image::RgbImage::new(1600, 1200);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgb([(x % 251) as u8, (y % 241) as u8, ((x + y) % 233) as u8]);
        }

        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        png
    }

    #[test]
    fn the_registered_pipeline_reproduces_the_golden_frames() {
        let png = source_png();

        let cropped = pipeline()
            .run(
                &png,
                &FrameContext {
                    crop_to: Some((1200, 1600)),
                    sleep_label: Some("zzz till 08:00".to_owned()),
                    palette: palette(),
                },
            )
            .unwrap();

        let rotated = pipeline()
            .run(
                &png,
                &FrameContext {
                    crop_to: None,
                    sleep_label: None,
                    palette: palette(),
                },
            )
            .unwrap();

        assert_eq!(cropped.len(), PACKED_FRAME_SIZE);
        assert_eq!(rotated.len(), PACKED_FRAME_SIZE);
        assert_eq!(crate::eink::image::content_hash(&cropped), GOLDEN_CROPPED);
        assert_eq!(crate::eink::image::content_hash(&rotated), GOLDEN_ROTATED);
    }

    #[test]
    fn an_inactive_step_contributes_nothing_to_the_fingerprint() {
        let bare = FrameContext {
            crop_to: None,
            sleep_label: None,
            palette: palette(),
        };

        let fingerprint = pipeline().fingerprint(&bare);

        assert!(!fingerprint.iter().any(|part| part.starts_with("crop=")));
        assert!(
            !fingerprint
                .iter()
                .any(|part| part.starts_with("sleep_label="))
        );
        assert!(fingerprint.iter().any(|part| part.starts_with("rotate=")));
        assert!(fingerprint.iter().any(|part| part.starts_with("pack=")));
    }

    #[test]
    fn a_changed_step_parameter_changes_the_fingerprint() {
        let pipeline = pipeline();

        let base = FrameContext {
            crop_to: Some((1200, 1600)),
            sleep_label: Some("zzz till 08:00".to_owned()),
            palette: palette(),
        };
        let relabelled = FrameContext {
            crop_to: base.crop_to,
            sleep_label: Some("zzz till 06:00".to_owned()),
            palette: base.palette.clone(),
        };
        let repainted = FrameContext {
            crop_to: base.crop_to,
            sleep_label: base.sleep_label.clone(),
            palette: vec![(0.0, 0.0, 0.0, 0), (250.0, 250.0, 250.0, 1)],
        };

        assert_eq!(pipeline.fingerprint(&base), pipeline.fingerprint(&base));
        assert_ne!(
            pipeline.fingerprint(&base),
            pipeline.fingerprint(&relabelled)
        );
        assert_ne!(
            pipeline.fingerprint(&base),
            pipeline.fingerprint(&repainted)
        );
    }

    #[test]
    fn an_unsupported_source_size_is_rejected() {
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(640, 480))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        let result = pipeline().run(
            &png,
            &FrameContext {
                crop_to: None,
                sleep_label: None,
                palette: palette(),
            },
        );

        assert!(result.is_err());
    }
}
