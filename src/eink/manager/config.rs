use super::EinkDisplayManager;
use super::resolve::ResolvedDisplay;
use crate::eink::partial::resolve_partial_window;
use crate::routes::epd::{DeviceReport, EpdConfig};
use chrono::Timelike;
use chrono_tz::Australia::Perth;

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
        let refresh_secs = match resolved.sleep {
            Some(sleep) => sleep.secs_until_end(chrono::Utc::now().with_timezone(&Perth).time()),
            None => aligned_refresh_secs(resolved.refresh_mins),
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

        if resolved.sleep.is_none() {
            config.refresh_interval_mins = Some(resolved.refresh_mins);

            if Some(resolved.firmware_version.as_str()) != report.running_firmware_version {
                config.firmware_version = Some(resolved.firmware_version.clone());
                config.firmware_url =
                    Some(format!("{HOST}/firmware?device_id={}", resolved.device_id));
            }
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
                    &self.db,
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

fn aligned_refresh_secs(mins: u32) -> u32 {
    let period = mins * 60;

    if period == 0 || 3600 % period != 0 {
        return period;
    }

    let now = chrono::Utc::now().with_timezone(&Perth).time();
    let secs_into_hour = now.minute() * 60 + now.second();

    period - (secs_into_hour % period)
}
