use sha2::{Digest, Sha256};

pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn content_type_for(key: &str) -> Option<&'static str> {
    let lower = key.to_ascii_lowercase();
    if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    }
}

pub fn center_crop_cover(img: &image::RgbImage, target_w: u32, target_h: u32) -> image::RgbImage {
    let (w, h) = img.dimensions();
    let scale = (target_w as f32 / w as f32).max(target_h as f32 / h as f32);
    let scaled_w = (w as f32 * scale).ceil() as u32;
    let scaled_h = (h as f32 * scale).ceil() as u32;
    let scaled = image::imageops::resize(
        img,
        scaled_w.max(target_w),
        scaled_h.max(target_h),
        image::imageops::FilterType::Lanczos3,
    );
    let (sw, sh) = scaled.dimensions();
    let x = (sw - target_w) / 2;
    let y = (sh - target_h) / 2;
    image::imageops::crop_imm(&scaled, x, y, target_w, target_h).to_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_crop_cover_matches_target_dims() {
        let src = image::RgbImage::from_pixel(800, 400, image::Rgb([10, 20, 30]));
        let out = center_crop_cover(&src, 1200, 1600);
        assert_eq!(out.dimensions(), (1200, 1600));

        let out = center_crop_cover(&src, 1600, 1200);
        assert_eq!(out.dimensions(), (1600, 1200));
    }

    #[test]
    fn content_hash_is_stable() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
        assert_ne!(content_hash(b"hello"), content_hash(b"world"));
    }
}
