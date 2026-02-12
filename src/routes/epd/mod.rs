use crate::types::{ApiState, AppError};
use axum::{Json, extract::State};
use open_feature::EvaluationContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub image_url: Option<String>,
    pub clear_screen: Option<bool>,
}

pub async fn config(
    State(ApiState {
        feature_flag_client,
        ..
    }): State<ApiState>,
) -> Json<EpdConfig> {
    Json(EpdConfig {
        refresh_interval_mins: Some(15),
        #[cfg(debug_assertions)]
        image_url: Some("http://192.168.0.104:8000/v1/epd/latest".to_string()),
        #[cfg(not(debug_assertions))]
        image_url: Some("https://home.anurag.sh/v1/epd/latest".to_string()),
        clear_screen: Some(
            feature_flag_client
                .is_feature_enabled(
                    "home-gateway-epd-clear-screen",
                    false,
                    EvaluationContext::default(),
                )
                .await,
        ),
    })
}

pub async fn latest(
    State(ApiState {
        object_registry, ..
    }): State<ApiState>,
) -> Result<Vec<u8>, AppError> {
    let image_response = object_registry
        .get_object::<Vec<u8>>("home-gateway", "image.png", None, false)
        .await?;

    let output_packed = tokio::task::spawn_blocking(move || {
        let mut img = image::load_from_memory(&image_response.payload)?.to_rgb8();
        let (width, height) = img.dimensions();

        if width == 1600 && height == 1200 {
            img = image::imageops::rotate90(&img);
        } else if width != 1200 || height != 1600 {
            return Err(anyhow::anyhow!(
                "Image dimensions must be 1600x1200 (will be rotated) or 1200x1600"
            ));
        }

        let (width, height) = img.dimensions();

        // Convert to floating point for error diffusion
        let mut buffer: Vec<f32> = Vec::with_capacity((width * height * 3) as usize);
        for pixel in img.pixels() {
            buffer.push(pixel[0] as f32);
            buffer.push(pixel[1] as f32);
            buffer.push(pixel[2] as f32);
        }

        let mut output_packed = Vec::with_capacity((width * height / 2) as usize);

        // Palette: Black, White, Yellow, Red, Blue, Green
        // Indices: 0, 1, 2, 3, 5, 6
        let palette = [
            (0.0, 0.0, 0.0, 0u8),       // Black
            (255.0, 255.0, 255.0, 1u8), // White
            (255.0, 255.0, 0.0, 2u8),   // Yellow
            (255.0, 0.0, 0.0, 3u8),     // Red
            (0.0, 0.0, 255.0, 5u8),     // Blue
            (0.0, 255.0, 0.0, 6u8),     // Green
        ];

        for y in 0..height {
            for x in (0..width).step_by(2) {
                // Process pixel 1 (high nibble)
                let idx1 = process_pixel(&mut buffer, width, height, x, y, &palette);

                // Process pixel 2 (low nibble)
                let idx2 = process_pixel(&mut buffer, width, height, x + 1, y, &palette);

                output_packed.push((idx1 << 4) | idx2);
            }
        }
        Ok(output_packed)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Join error: {}", e))??;

    Ok(output_packed)
}

fn process_pixel(
    buffer: &mut [f32],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    palette: &[(f32, f32, f32, u8)],
) -> u8 {
    let index = ((y * width + x) * 3) as usize;
    let r = buffer[index];
    let g = buffer[index + 1];
    let b = buffer[index + 2];

    // Find closest color
    let mut min_dist = f32::MAX;
    let mut closest_idx = 0;
    let mut closest_color = (0.0, 0.0, 0.0);

    for &(pr, pg, pb, pidx) in palette {
        let dr = r - pr;
        let dg = g - pg;
        let db = b - pb;
        let dist = dr * dr + dg * dg + db * db;
        if dist < min_dist {
            min_dist = dist;
            closest_idx = pidx;
            closest_color = (pr, pg, pb);
        }
    }

    let (pr, pg, pb) = closest_color;
    let err_r = r - pr;
    let err_g = g - pg;
    let err_b = b - pb;

    // Distribute error
    // (x+1, y) 7/16
    if x + 1 < width {
        add_error(buffer, width, x + 1, y, err_r, err_g, err_b, 7.0 / 16.0);
    }
    // (x-1, y+1) 3/16
    if x > 0 && y + 1 < height {
        add_error(buffer, width, x - 1, y + 1, err_r, err_g, err_b, 3.0 / 16.0);
    }
    // (x, y+1) 5/16
    if y + 1 < height {
        add_error(buffer, width, x, y + 1, err_r, err_g, err_b, 5.0 / 16.0);
    }
    // (x+1, y+1) 1/16
    if x + 1 < width && y + 1 < height {
        add_error(buffer, width, x + 1, y + 1, err_r, err_g, err_b, 1.0 / 16.0);
    }

    closest_idx
}

fn add_error(
    buffer: &mut [f32],
    width: u32,
    x: u32,
    y: u32,
    er: f32,
    eg: f32,
    eb: f32,
    factor: f32,
) {
    let index = ((y * width + x) * 3) as usize;
    buffer[index] += er * factor;
    buffer[index + 1] += eg * factor;
    buffer[index + 2] += eb * factor;
}
