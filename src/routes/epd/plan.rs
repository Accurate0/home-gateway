use super::{panel::packed_cache_key, render::render_and_cache};
use crate::epd::{EpdFlagConfig, epd_palette};
use crate::{
    device_registry::DeviceRegistry,
    settings::SleepWindow,
    feature_flag::FeatureFlagClient,
    settings::SettingsContainer,
    error::AppError,
};
use chrono_tz::Australia::Perth;
use sha2::{Digest, Sha256};

const SLEEP_IMAGE_PREFIX: &str = "eink-display/sleep/";
const RENDER_PIPELINE_VERSION: u32 = 2;

pub(super) struct RenderPlan {
    image_key: String,
    pub(super) sleep: Option<SleepWindow>,
    sleep_label: Option<String>,
    palette: Vec<(f32, f32, f32, u8)>,
    pub(super) hash: String,
}

pub(super) async fn render_plan(
    db: &sqlx::Pool<sqlx::Postgres>,
    s3: &crate::s3::S3,
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    settings: &SettingsContainer,
    flag: &EpdFlagConfig,
    device_id: &str,
) -> Option<RenderPlan> {
    let palette = epd_palette(
        feature_flag_client,
        devices,
        &settings.eink_display.palette,
        device_id,
    )
    .await;

    let palette_hash = palette
        .iter()
        .map(|(r, g, b, index)| format!("{r}:{g}:{b}:{index}"))
        .collect::<Vec<_>>()
        .join(",");

    let sleep = active_sleep(devices, device_id, flag.force_sleep);
    let now = chrono::Utc::now().with_timezone(&Perth).naive_local();

    let (image_key, content_hash, sleep_label) = match sleep {
        Some(sleep) => {
            let images = s3
                .list_objects(SLEEP_IMAGE_PREFIX)
                .await
                .unwrap_or_default();
            let seed = format!("{}/{}", sleep.end, window_start_date(&sleep, now));

            match pick_sleep_image(&images, &seed) {
                Some(image_key) => {
                    let label = format!("zzz till {}", sleep.end.format("%H:%M"));
                    (image_key.clone(), image_key, Some(label))
                }
                None => {
                    tracing::warn!(
                        "no sleep images under {SLEEP_IMAGE_PREFIX}, serving the latest render"
                    );
                    let (image_key, content_hash) = latest_image(db, device_id).await.ok()?;
                    (image_key, content_hash, None)
                }
            }
        }
        None => {
            let (image_key, content_hash) = latest_image(db, device_id).await.ok()?;
            (image_key, content_hash, None)
        }
    };

    let hash = render_hash(&[
        &RENDER_PIPELINE_VERSION.to_string(),
        &image_key,
        &content_hash,
        &palette_hash,
        &flag.clear_screen.to_string(),
        sleep_label.as_deref().unwrap_or(""),
    ]);

    Some(RenderPlan {
        image_key,
        sleep,
        sleep_label,
        palette,
        hash,
    })
}

pub(super) async fn ensure_packed_cached(s3: &crate::s3::S3, plan: &RenderPlan) -> bool {
    let key = packed_cache_key(plan.hash.clone());

    match s3.get_object_metadata(&key).await {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(key = %key, "failed to check the packed cache: {e}");
            return false;
        }
    }

    let result = render_and_cache(
        s3,
        &plan.image_key,
        plan.sleep_label.clone(),
        &plan.palette,
        &key,
    )
    .await;

    match result {
        Ok(_) => true,
        Err(_) => {
            tracing::warn!(key = %key, "failed to prepare the packed frame");
            false
        }
    }
}

fn active_sleep(
    devices: &DeviceRegistry,
    device_id: &str,
    force_sleep: bool,
) -> Option<SleepWindow> {
    let sleep = devices.eink_display(device_id).and_then(|d| d.sleep)?;
    let now = chrono::Utc::now().with_timezone(&Perth).time();
    if sleep.contains(now) || force_sleep {
        return Some(sleep);
    }
    None
}

async fn latest_image(
    db: &sqlx::Pool<sqlx::Postgres>,
    device_id: &str,
) -> Result<(String, String), AppError> {
    let row = sqlx::query!(
        "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
        device_id
    )
    .fetch_optional(db)
    .await?;

    let image_key = row
        .as_ref()
        .and_then(|row| row.image_key.clone())
        .ok_or_else(|| {
            AppError::Error(anyhow::anyhow!("no rendered image for device {device_id}"))
        })?;

    let content_hash = row
        .and_then(|row| row.image_content_hash)
        .unwrap_or_default();

    Ok((image_key, content_hash))
}

fn window_start_date(sleep: &SleepWindow, now: chrono::NaiveDateTime) -> chrono::NaiveDate {
    if now.time() >= sleep.start {
        now.date()
    } else {
        now.date().pred_opt().unwrap_or(now.date())
    }
}

fn pick_sleep_image(images: &[String], seed: &str) -> Option<String> {
    if images.is_empty() {
        return None;
    }

    let mut sorted = images.to_vec();
    sorted.sort();

    let digest = Sha256::digest(seed.as_bytes());
    let index = u64::from_be_bytes(digest[..8].try_into().ok()?) % sorted.len() as u64;

    sorted.into_iter().nth(index as usize)
}

fn render_hash(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }

    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_hash_is_stable() {
        assert_eq!(
            render_hash(&["1", "key.png"]),
            render_hash(&["1", "key.png"])
        );
    }

    #[test]
    fn render_hash_changes_with_inputs() {
        let base = render_hash(&["1", "key.png", "false"]);

        assert_ne!(base, render_hash(&["2", "key.png", "false"]));
        assert_ne!(base, render_hash(&["1", "other.png", "false"]));
        assert_ne!(base, render_hash(&["1", "key.png", "true"]));
    }

    #[test]
    fn render_hash_separates_parts() {
        assert_ne!(render_hash(&["ab", "c"]), render_hash(&["a", "bc"]));
    }

    fn sleep_images() -> Vec<String> {
        (0..8).map(|i| format!("sleep/{i}.png")).collect()
    }

    #[test]
    fn a_sleep_pick_is_stable_for_the_same_seed() {
        let images = sleep_images();
        assert_eq!(
            pick_sleep_image(&images, "08:00/2026-08-04"),
            pick_sleep_image(&images, "08:00/2026-08-04")
        );
    }

    #[test]
    fn a_sleep_pick_ignores_listing_order() {
        let images = sleep_images();
        let mut reversed = images.clone();
        reversed.reverse();

        assert_eq!(
            pick_sleep_image(&images, "08:00/2026-08-04"),
            pick_sleep_image(&reversed, "08:00/2026-08-04")
        );
    }

    #[test]
    fn a_sleep_pick_varies_across_nights() {
        let images = sleep_images();
        let picks = (1..=10)
            .map(|day| pick_sleep_image(&images, &format!("08:00/2026-08-{day:02}")))
            .collect::<std::collections::HashSet<_>>();

        assert!(picks.len() > 1, "every night picked the same image");
    }

    #[test]
    fn an_empty_sleep_listing_picks_nothing() {
        assert_eq!(pick_sleep_image(&[], "08:00/2026-08-04"), None);
    }

    #[test]
    fn a_sleep_window_that_straddles_midnight_keeps_one_start_date() {
        let sleep = SleepWindow {
            start: chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        };

        let evening = chrono::NaiveDate::from_ymd_opt(2026, 8, 4)
            .unwrap()
            .and_hms_opt(23, 0, 0)
            .unwrap();
        let morning = chrono::NaiveDate::from_ymd_opt(2026, 8, 5)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();

        assert_eq!(
            window_start_date(&sleep, evening),
            window_start_date(&sleep, morning)
        );
    }
}
