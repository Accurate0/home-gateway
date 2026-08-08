use crate::eink::manager::resolve::ResolvedDisplay;
use crate::integrations::s3::S3;
use crate::settings::SleepWindow;
use chrono_tz::Australia::Perth;
use sha2::{Digest, Sha256};

const SLEEP_IMAGE_PREFIX: &str = "eink-display/sleep/";

pub struct SleepSource;

impl SleepSource {
    pub async fn acquire(
        &self,
        s3: &S3,
        resolved: &ResolvedDisplay,
        sleep: SleepWindow,
    ) -> Option<String> {
        let images = s3
            .list_objects(SLEEP_IMAGE_PREFIX)
            .await
            .unwrap_or_default();
        let now = chrono::Utc::now().with_timezone(&Perth).naive_local();
        let seed = format!("{}/{}", sleep.end, window_start_date(&sleep, now));

        let picked = pick_sleep_image(&images, &seed);

        if picked.is_none() {
            tracing::warn!(
                device_id = resolved.device_id,
                "no sleep images under {SLEEP_IMAGE_PREFIX}, serving the latest render"
            );
        }

        picked
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
            grace: chrono::TimeDelta::minutes(5),
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
