use crate::eink::flag::EpdFlagConfig;
use crate::settings::{
    Album, DashboardView, EinkDisplaySettings, EinkGlobalSettings, EinkMode, Orientation,
    PartialRefresh, RedditFeed, RedditTimespan, SleepWindow,
};
use chrono_tz::Australia::Perth;
use std::time::Duration;

const DEFAULT_REFRESH_INTERVAL_MINS: u32 = 15;
const DEFAULT_REDDIT_TIMESPAN: RedditTimespan = RedditTimespan::Day;
const DEFAULT_REDDIT_LIMIT: u32 = 25;
const DEFAULT_SETTLE: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct ResolvedDisplay {
    pub device_id: String,
    pub name: String,
    pub mode: EinkMode,
    pub refresh_mins: u32,
    pub orientation: Orientation,
    pub view: Option<DashboardView>,
    pub album: Album,
    pub feed: Option<RedditFeed>,
    pub sleep: Option<SleepWindow>,
    pub lead: Option<chrono::TimeDelta>,
    pub settle: Duration,
    pub palette: Vec<(f32, f32, f32, u8)>,
    pub partial: PartialRefresh,
    pub partial_enabled: bool,
    pub clear_screen: bool,
    pub firmware_version: String,
}

impl ResolvedDisplay {
    pub fn resolve(
        device_id: &str,
        display: &EinkDisplaySettings,
        global: &EinkGlobalSettings,
        flag: &EpdFlagConfig,
        palette: Vec<(f32, f32, f32, u8)>,
    ) -> Self {
        let mode = flag.mode.unwrap_or_else(|| display.mode.name());

        Self {
            device_id: device_id.to_owned(),
            name: display.name.clone(),
            mode,
            refresh_mins: refresh_mins(flag, Some(display)),
            orientation: display.orientation,
            view: view(flag, global, display).cloned(),
            album: album(flag, global, display),
            feed: feed(flag, display),
            sleep: active_sleep(display, flag.force_sleep, device_id),
            lead: display.mode.lead(),
            settle: display
                .mode
                .settle()
                .and_then(|settle| settle.to_std().ok())
                .unwrap_or(DEFAULT_SETTLE),
            palette,
            partial: display.partial,
            partial_enabled: flag.partial_refresh.unwrap_or(display.partial.enabled),
            clear_screen: flag.clear_screen,
            firmware_version: flag
                .firmware_version
                .clone()
                .unwrap_or_else(|| display.firmware_version.clone()),
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

pub fn refresh_mins(flag: &EpdFlagConfig, display: Option<&EinkDisplaySettings>) -> u32 {
    if let Some(interval) = flag.refresh_interval {
        return interval;
    }

    display
        .and_then(|display| display.refresh)
        .map(|refresh| refresh.num_minutes().max(1) as u32)
        .unwrap_or(DEFAULT_REFRESH_INTERVAL_MINS)
}

fn view<'a>(
    flag: &EpdFlagConfig,
    global: &'a EinkGlobalSettings,
    display: &EinkDisplaySettings,
) -> Option<&'a DashboardView> {
    let name = flag
        .dashboard_view
        .clone()
        .or_else(|| display.mode.view().map(str::to_owned))?;

    global.view(&name).or_else(|| {
        tracing::warn!("dashboard view `{name}` not configured, ignoring");
        None
    })
}

fn album(
    flag: &EpdFlagConfig,
    global: &EinkGlobalSettings,
    display: &EinkDisplaySettings,
) -> Album {
    let name = flag
        .album
        .clone()
        .or_else(|| display.mode.album().map(str::to_owned));

    let Some(name) = name else {
        return global.default_album();
    };

    global.album(&name).cloned().unwrap_or_else(|| {
        tracing::warn!("album `{name}` not configured, using default prefix");
        global.default_album()
    })
}

fn feed(flag: &EpdFlagConfig, display: &EinkDisplaySettings) -> Option<RedditFeed> {
    let configured = display.mode.feed();

    let subreddit = flag
        .subreddit
        .clone()
        .or_else(|| configured.map(|feed| feed.subreddit.clone()))?;

    Some(RedditFeed {
        subreddit,
        timespan: flag
            .timespan
            .or_else(|| configured.map(|feed| feed.timespan))
            .unwrap_or(DEFAULT_REDDIT_TIMESPAN),
        limit: flag
            .limit
            .or_else(|| configured.map(|feed| feed.limit))
            .unwrap_or(DEFAULT_REDDIT_LIMIT),
    })
}

fn active_sleep(
    display: &EinkDisplaySettings,
    force_sleep: bool,
    device_id: &str,
) -> Option<SleepWindow> {
    let sleep = display.sleep?;

    if force_sleep {
        return Some(sleep);
    }

    let now = chrono::Utc::now().with_timezone(&Perth).time();

    if !sleep.contains(now) {
        return None;
    }

    if sleep.ending_within_grace(now) {
        tracing::info!(
            device_id = %device_id,
            remaining_secs = sleep.secs_until_end(now),
            "woke inside the sleep grace, serving the dashboard instead of another sleep cycle"
        );
        return None;
    }

    Some(sleep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::EinkModeConfig;
    use pretty_assertions::assert_eq;

    fn display_with_refresh(refresh: Option<chrono::TimeDelta>) -> EinkDisplaySettings {
        EinkDisplaySettings {
            name: "Test Display".to_owned(),
            firmware_version: "v0.0.0".to_owned(),
            mode: EinkModeConfig::Dashboard {
                view: None,
                settle: chrono::TimeDelta::seconds(10),
                lead: chrono::TimeDelta::minutes(15),
            },
            orientation: Orientation::Portrait,
            refresh,
            sleep: None,
            partial: PartialRefresh {
                enabled: false,
                max_area_pct: 30,
                max_consecutive: 5,
            },
        }
    }

    fn display_with_feed(feed: RedditFeed) -> EinkDisplaySettings {
        EinkDisplaySettings {
            mode: EinkModeConfig::Reddit { feed },
            ..display_with_refresh(None)
        }
    }

    #[test]
    fn reddit_feed_falls_back_to_the_display() {
        let configured = RedditFeed {
            subreddit: "EarthPorn".to_owned(),
            timespan: RedditTimespan::Week,
            limit: 40,
        };

        assert_eq!(
            feed(
                &EpdFlagConfig::default(),
                &display_with_feed(configured.clone())
            ),
            Some(configured)
        );
    }

    #[test]
    fn reddit_feed_prefers_the_flag_per_field() {
        let flag = EpdFlagConfig {
            subreddit: Some("ImaginaryLandscapes".to_owned()),
            timespan: Some(RedditTimespan::Day),
            ..EpdFlagConfig::default()
        };
        let display = display_with_feed(RedditFeed {
            subreddit: "EarthPorn".to_owned(),
            timespan: RedditTimespan::Week,
            limit: 40,
        });

        assert_eq!(
            feed(&flag, &display),
            Some(RedditFeed {
                subreddit: "ImaginaryLandscapes".to_owned(),
                timespan: RedditTimespan::Day,
                limit: 40,
            })
        );
    }

    #[test]
    fn reddit_feed_without_a_subreddit_is_skipped() {
        assert_eq!(
            feed(&EpdFlagConfig::default(), &display_with_refresh(None)),
            None
        );
    }

    #[test]
    fn reddit_feed_from_the_flag_alone_uses_defaults() {
        let flag = EpdFlagConfig {
            subreddit: Some("EarthPorn".to_owned()),
            ..EpdFlagConfig::default()
        };

        assert_eq!(
            feed(&flag, &display_with_refresh(None)),
            Some(RedditFeed {
                subreddit: "EarthPorn".to_owned(),
                timespan: DEFAULT_REDDIT_TIMESPAN,
                limit: DEFAULT_REDDIT_LIMIT,
            })
        );
    }

    #[test]
    fn refresh_interval_prefers_the_flag() {
        let flag = EpdFlagConfig {
            refresh_interval: Some(5),
            ..EpdFlagConfig::default()
        };
        let display = display_with_refresh(Some(chrono::TimeDelta::hours(1)));

        assert_eq!(refresh_mins(&flag, Some(&display)), 5);
    }

    #[test]
    fn refresh_interval_falls_back_to_the_display() {
        let flag = EpdFlagConfig::default();

        assert_eq!(
            refresh_mins(
                &flag,
                Some(&display_with_refresh(Some(chrono::TimeDelta::hours(1))))
            ),
            60
        );

        assert_eq!(
            refresh_mins(
                &flag,
                Some(&display_with_refresh(Some(chrono::TimeDelta::seconds(30))))
            ),
            1
        );

        assert_eq!(
            refresh_mins(&flag, Some(&display_with_refresh(None))),
            DEFAULT_REFRESH_INTERVAL_MINS
        );

        assert_eq!(refresh_mins(&flag, None), DEFAULT_REFRESH_INTERVAL_MINS);
    }
}
