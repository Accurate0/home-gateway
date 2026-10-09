use super::EinkDisplayManager;
use super::resolve::ResolvedDisplay;
use crate::cron::CronSchedule;
use crate::eink::panel::PartialWindow;
use crate::eink::partial::resolve_partial_window;
use crate::routes::epd::DeviceReport;
use crate::settings::EinkDefaults;
use chrono::{DateTime, TimeDelta};
use chrono_tz::Australia::Perth;
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct PlannedFrame {
    pub hash: String,
    pub packed: bytes::Bytes,
    pub partial: Option<PartialWindow>,
}

#[derive(Debug, Clone)]
pub struct WakeDecision {
    pub refresh_secs: u32,
    pub firmware_version: Option<String>,
    pub clear_screen: bool,
    pub frame: Option<PlannedFrame>,
}

impl EinkDisplayManager {
    #[tracing::instrument(
        name = "eink.wake_decision",
        skip_all,
        fields(device_id = %resolved.device_id)
    )]
    pub async fn wake_decision(
        &self,
        resolved: &ResolvedDisplay,
        report: DeviceReport<'_>,
    ) -> WakeDecision {
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

        let wants_firmware = resolved.sleep.is_none()
            && Some(resolved.firmware_version.as_str()) != report.running_firmware_version;

        let mut decision = WakeDecision {
            refresh_secs,
            firmware_version: wants_firmware.then(|| resolved.firmware_version.clone()),
            clear_screen: resolved.clear_screen,
            frame: None,
        };

        let wants_partial = resolved.wants_partial();

        let previous = async {
            match report.current_image_hash.filter(|_| wants_partial) {
                Some(hash) => self.packed_frame(hash).await,
                None => None,
            }
        };

        let planned = async {
            let plan = self.plan(resolved).await?;
            let packed = self.ensure_packed(&plan, None).await?;

            Some((plan, packed))
        };

        let (previous, planned) = tokio::join!(previous, planned);

        let Some((plan, packed)) = planned else {
            tracing::warn!(
                device_id = resolved.device_id,
                "no image to serve, skipping this cycle"
            );
            return decision;
        };

        let partial = match resolved.clear_screen || plan.sleep.is_some() {
            true => None,
            false => {
                resolve_partial_window(
                    &self.eink,
                    resolved,
                    report.current_image_hash,
                    &plan.hash,
                    previous.as_deref(),
                    &packed,
                )
                .await
            }
        };

        decision.frame = Some(PlannedFrame {
            hash: plan.hash,
            packed,
            partial,
        });

        decision
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
