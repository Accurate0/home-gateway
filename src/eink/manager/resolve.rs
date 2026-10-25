use crate::cron::CronSchedule;
use crate::settings::{
    DashboardView, EinkDisplaySettings, EinkGlobalSettings, EinkMode, Orientation, PartialRefresh,
    RedditFeed, SleepWindow,
};
use chrono_tz::Australia::Perth;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResolvedDisplay {
    pub device_id: String,
    pub name: String,
    pub mode: EinkMode,
    pub refresh: CronSchedule,
    pub grace: chrono::TimeDelta,
    pub orientation: Orientation,
    pub view: Option<DashboardView>,
    pub feed: Option<RedditFeed>,
    pub immich_album: Option<String>,
    pub sleep: Option<SleepWindow>,
    pub lead: Option<chrono::TimeDelta>,
    pub settle: Duration,
    pub partial: PartialRefresh,
    pub firmware_version: String,
}

impl ResolvedDisplay {
    pub fn resolve(
        device_id: &str,
        display: &EinkDisplaySettings,
        global: &EinkGlobalSettings,
    ) -> Self {
        Self {
            device_id: device_id.to_owned(),
            name: display.name.clone(),
            mode: display.mode.name(),
            refresh: display.refresh.clone(),
            grace: display.grace,
            orientation: display.orientation,
            view: view(global, display).cloned(),
            feed: display.mode.feed().cloned(),
            immich_album: display.mode.immich_album().map(str::to_owned),
            sleep: active_sleep(display, device_id),
            lead: display.mode.lead(),
            settle: display
                .mode
                .settle()
                .and_then(|settle| settle.to_std().ok())
                .unwrap_or_else(|| global.defaults.settle()),
            partial: display.partial,
            firmware_version: global.firmware_version.clone(),
        }
    }

    pub fn target_dims(&self) -> (u32, u32) {
        self.orientation.target_dims()
    }

    pub fn orientation_str(&self) -> &'static str {
        self.orientation.as_str()
    }

    pub fn sleep_label(&self) -> Option<String> {
        self.sleep
            .map(|sleep| format!("zzz till {}", sleep.end.format("%H:%M")))
    }
}

fn view<'a>(
    global: &'a EinkGlobalSettings,
    display: &EinkDisplaySettings,
) -> Option<&'a DashboardView> {
    let name = display.mode.view()?;

    global.view(name).or_else(|| {
        tracing::warn!("dashboard view `{name}` not configured, ignoring");
        None
    })
}

fn active_sleep(display: &EinkDisplaySettings, device_id: &str) -> Option<SleepWindow> {
    let sleep = display.sleep?;

    let now = chrono::Utc::now().with_timezone(&Perth).time();

    if !sleep.contains(now) {
        return None;
    }

    if sleep.ending_within(now, display.grace) {
        tracing::info!(
            device_id = %device_id,
            remaining_secs = sleep.secs_until_end(now),
            "woke inside the sleep grace, serving the dashboard instead of another sleep cycle"
        );
        return None;
    }

    Some(sleep)
}
