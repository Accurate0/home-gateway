use super::EinkDisplayManager;
use super::resolve::ResolvedDisplay;
use crate::actors::system::cron::schedule::CronSchedule;
use crate::eink::partial::resolve_partial_window;
use crate::routes::epd::{DeviceReport, EpdConfig};
use crate::settings::EinkDefaults;
use chrono::{DateTime, TimeDelta};
use chrono_tz::Australia::Perth;
use chrono_tz::Tz;

#[cfg(debug_assertions)]
const HOST: &str = "http://192.168.0.149:8000/v1/epd";
#[cfg(not(debug_assertions))]
const HOST: &str = "https://home.anurag.sh/v1/epd";

impl EinkDisplayManager {
    pub async fn epd_config(
        &self,
        resolved: &ResolvedDisplay,
        report: DeviceReport<'_>,
    ) -> EpdConfig {
        let now = chrono::Utc::now().with_timezone(&Perth);

        let refresh_secs = match resolved.sleep {
            Some(sleep) => sleep.secs_until_end(now.time()),
            None => drift_biased_refresh_secs(
                &resolved.refresh,
                resolved.grace,
                now,
                &self.settings.eink_display.defaults,
            ),
        };

        let mut config = EpdConfig {
            refresh_interval_mins: Some(refresh_secs.div_ceil(60)),
            refresh_interval_secs: Some(refresh_secs),
            image_url: None,
            image_hash: None,
            clear_screen: Some(resolved.clear_screen),
            firmware_url: None,
            firmware_version: None,
            partial: None,
        };

        if resolved.sleep.is_none()
            && Some(resolved.firmware_version.as_str()) != report.running_firmware_version
        {
            config.firmware_version = Some(resolved.firmware_version.clone());
            config.firmware_url = Some(format!("{HOST}/firmware?device_id={}", resolved.device_id));
        }

        let Some(plan) = self.plan(resolved).await else {
            tracing::warn!(
                device_id = resolved.device_id,
                "no image to serve, skipping this cycle"
            );
            return config;
        };

        if !self.ensure_packed(&plan).await {
            return config;
        }

        let partial = match resolved.clear_screen || plan.sleep.is_some() {
            true => None,
            false => {
                resolve_partial_window(
                    &self.eink,
                    &self.s3,
                    resolved,
                    report.current_image_hash,
                    &plan.hash,
                )
                .await
            }
        };

        let mut url = format!(
            "{HOST}/image/{}?device_id={}",
            plan.hash, resolved.device_id
        );

        if let Some(window) = partial {
            url.push_str(&format!(
                "&x={}&y={}&width={}&height={}",
                window.x, window.y, window.width, window.height
            ));
        }

        config.image_url = Some(url);
        config.image_hash = Some(plan.hash);
        config.partial = partial;

        config
    }
}

fn drift_biased_refresh_secs(
    refresh: &CronSchedule,
    grace: TimeDelta,
    now: DateTime<Tz>,
    defaults: &EinkDefaults,
) -> u32 {
    let secs = match refresh.secs_until_next_from(now, grace) {
        Ok(secs) => secs,
        Err(e) => {
            tracing::error!(
                "refresh schedule `{}` has no next occurrence ({e}), retrying shortly",
                refresh.expression()
            );
            return defaults.fallback_refresh_secs();
        }
    };

    let bias = (grace.num_seconds().max(0) / 2) as u32;

    secs.saturating_sub(bias).max(defaults.min_refresh_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    fn hourly() -> CronSchedule {
        CronSchedule::parse("0 * * * *").unwrap()
    }

    fn defaults() -> EinkDefaults {
        EinkDefaults {
            reddit_limit: 25,
            settle: TimeDelta::seconds(10),
            fallback_refresh: TimeDelta::minutes(15),
            min_refresh: TimeDelta::seconds(60),
        }
    }

    fn perth(hour: u32, minute: u32) -> DateTime<Tz> {
        Perth
            .with_ymd_and_hms(2026, 8, 16, hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn a_wake_sleeps_to_just_before_the_next_slot() {
        assert_eq!(
            drift_biased_refresh_secs(&hourly(), TimeDelta::minutes(10), perth(10, 0), &defaults()),
            55 * 60
        );
    }

    #[test]
    fn an_early_wake_holds_the_same_phase_rather_than_compounding() {
        assert_eq!(
            drift_biased_refresh_secs(
                &hourly(),
                TimeDelta::minutes(10),
                perth(10, 55),
                &defaults()
            ),
            60 * 60
        );
    }

    #[test]
    fn a_late_wake_corrects_back_onto_the_phase() {
        assert_eq!(
            drift_biased_refresh_secs(&hourly(), TimeDelta::minutes(10), perth(11, 2), &defaults()),
            53 * 60
        );
    }

    #[test]
    fn a_zero_grace_targets_the_slot_exactly() {
        assert_eq!(
            drift_biased_refresh_secs(&hourly(), TimeDelta::zero(), perth(10, 30), &defaults()),
            30 * 60
        );
    }
}
