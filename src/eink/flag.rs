use crate::{
    device_registry::DeviceRegistry,
    settings::EinkDisplaySettings,
    feature_flag::FeatureFlagClient,
    settings::{
        Album, DashboardView, EinkGlobalSettings, EinkMode, PaletteColor, RedditFeed,
        RedditTimespan,
    },
};
use open_feature::EvaluationContext;

const EPD_FLAG: &str = "home-gateway-epd";
const EPD_DITHERING_CONFIG_FLAG: &str = "home-gateway-epd-dithering-config";

const DEFAULT_REFRESH_INTERVAL_MINS: u32 = 15;
const DEFAULT_REDDIT_TIMESPAN: RedditTimespan = RedditTimespan::Day;
const DEFAULT_REDDIT_LIMIT: u32 = 25;

#[derive(Debug, Clone, Default)]
pub struct EpdFlagConfig {
    pub clear_screen: bool,
    pub force_sleep: bool,
    refresh_interval: Option<u32>,
    mode: Option<EinkMode>,
    dashboard_view: Option<String>,
    album: Option<String>,
    subreddit: Option<String>,
    timespan: Option<RedditTimespan>,
    limit: Option<u32>,
    firmware_version: Option<String>,
    pub partial_refresh: Option<bool>,
}

impl From<open_feature::StructValue> for EpdFlagConfig {
    fn from(value: open_feature::StructValue) -> Self {
        let default = EpdFlagConfig::default();
        let string_field = |key: &str| {
            value
                .fields
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        };
        Self {
            clear_screen: value
                .fields
                .get("clear_screen")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.clear_screen),
            force_sleep: value
                .fields
                .get("force_sleep")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.force_sleep),
            refresh_interval: value
                .fields
                .get("refresh_interval")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|n| n.max(1) as u32),
            mode: string_field("mode").and_then(|s| match s.as_str() {
                "dashboard" => Some(EinkMode::Dashboard),
                "album" => Some(EinkMode::Album),
                "reddit" => Some(EinkMode::Reddit),
                other => {
                    tracing::warn!("unknown epd mode `{other}` in flag, ignoring");
                    None
                }
            }),
            dashboard_view: string_field("dashboard_view"),
            album: string_field("album"),
            subreddit: string_field("subreddit"),
            timespan: string_field("timespan").and_then(|s| match s.as_str() {
                "hour" => Some(RedditTimespan::Hour),
                "day" => Some(RedditTimespan::Day),
                "week" => Some(RedditTimespan::Week),
                "month" => Some(RedditTimespan::Month),
                "year" => Some(RedditTimespan::Year),
                "all" => Some(RedditTimespan::All),
                other => {
                    tracing::warn!("unknown reddit timespan `{other}` in flag, ignoring");
                    None
                }
            }),
            limit: value
                .fields
                .get("limit")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|n| n.clamp(1, 100) as u32),
            firmware_version: string_field("firmware_version"),
            partial_refresh: value
                .fields
                .get("partial_refresh")
                .and_then(|v| v.as_bool()),
        }
    }
}

impl EpdFlagConfig {
    pub fn resolve_refresh_interval(&self, display: Option<&EinkDisplaySettings>) -> u32 {
        if let Some(interval) = self.refresh_interval {
            return interval;
        }

        display
            .and_then(|display| display.refresh)
            .map(|refresh| refresh.num_minutes().max(1) as u32)
            .unwrap_or(DEFAULT_REFRESH_INTERVAL_MINS)
    }

    pub fn resolve_mode(&self, display: &EinkDisplaySettings) -> EinkMode {
        self.mode.unwrap_or_else(|| display.mode.name())
    }

    pub fn resolve_view<'a>(
        &self,
        global: &'a EinkGlobalSettings,
        display: &EinkDisplaySettings,
    ) -> Option<&'a DashboardView> {
        let name = self
            .dashboard_view
            .clone()
            .or_else(|| display.mode.view().map(|v| v.to_owned()));
        match name {
            Some(name) => global.view(&name).or_else(|| {
                tracing::warn!("dashboard view `{name}` not configured, ignoring");
                None
            }),
            None => None,
        }
    }

    pub fn resolve_reddit_feed(&self, display: &EinkDisplaySettings) -> Option<RedditFeed> {
        let configured = display.mode.feed();

        let subreddit = self
            .subreddit
            .clone()
            .or_else(|| configured.map(|feed| feed.subreddit.clone()));
        let Some(subreddit) = subreddit else {
            let name = &display.name;
            tracing::warn!(
                "reddit mode selected for `{name}` but no subreddit is configured, skipping render"
            );
            return None;
        };

        Some(RedditFeed {
            subreddit,
            timespan: self
                .timespan
                .or_else(|| configured.map(|feed| feed.timespan))
                .unwrap_or(DEFAULT_REDDIT_TIMESPAN),
            limit: self
                .limit
                .or_else(|| configured.map(|feed| feed.limit))
                .unwrap_or(DEFAULT_REDDIT_LIMIT),
        })
    }

    pub fn resolve_album(
        &self,
        global: &EinkGlobalSettings,
        display: &EinkDisplaySettings,
    ) -> Album {
        let name = self
            .album
            .clone()
            .or_else(|| display.mode.album().map(|a| a.to_owned()));
        match name {
            Some(name) => global.album(&name).cloned().unwrap_or_else(|| {
                tracing::warn!("album `{name}` not configured, using default prefix");
                global.default_album()
            }),
            None => global.default_album(),
        }
    }
}

fn evaluation_context(devices: &DeviceRegistry, device_id: &str) -> EvaluationContext {
    let context =
        EvaluationContext::default().with_custom_field("device_id", device_id.to_string());

    match devices.eink_display(device_id) {
        Some(display) => context.with_custom_field("device_name", display.name.clone()),
        None => context,
    }
}

pub async fn epd_flag_config(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    device_id: &str,
) -> EpdFlagConfig {
    match feature_flag_client
        .get_struct(EPD_FLAG, evaluation_context(devices, device_id))
        .await
    {
        Ok(value) => {
            let config = EpdFlagConfig::from(value);
            tracing::info!("{EPD_FLAG} evaluated for {device_id}: {config:?}");
            config
        }
        Err(e) => {
            let fallback = EpdFlagConfig::default();
            tracing::error!(
                "error evaluating {EPD_FLAG} for {device_id}: {e:?}, using fallback: {fallback:?}"
            );
            fallback
        }
    }
}

pub async fn epd_palette(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    base: &[PaletteColor],
    device_id: &str,
) -> Vec<(f32, f32, f32, u8)> {
    let config = feature_flag_client
        .get_struct(
            EPD_DITHERING_CONFIG_FLAG,
            evaluation_context(devices, device_id),
        )
        .await
        .ok();

    base.iter()
        .map(|color| {
            let (r, g, b) = config
                .as_ref()
                .and_then(|c| c.fields.get(color.name))
                .and_then(|v| v.as_struct())
                .map(|s| {
                    let channel = |key: &str, default: f32| {
                        s.fields
                            .get(key)
                            .and_then(|v| {
                                v.as_i64()
                                    .map(|n| n as f32)
                                    .or_else(|| v.as_f64().map(|f| f as f32))
                            })
                            .map(|n| n.clamp(0.0, 255.0))
                            .unwrap_or(default)
                    };
                    (
                        channel("r", color.r),
                        channel("g", color.g),
                        channel("b", color.b),
                    )
                })
                .unwrap_or((color.r, color.g, color.b));
            (r, g, b, color.index)
        })
        .collect()
}

pub fn target_firmware_version(
    flag: &EpdFlagConfig,
    devices: &DeviceRegistry,
    device_id: &str,
) -> Option<String> {
    if let Some(override_version) = &flag.firmware_version {
        return Some(override_version.clone());
    }

    devices
        .eink_display(device_id)
        .map(|display| display.firmware_version.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display_with_refresh(refresh: Option<chrono::TimeDelta>) -> EinkDisplaySettings {
        EinkDisplaySettings {
            name: "Test Display".to_owned(),
            firmware_version: "v0.0.0".to_owned(),
            mode: crate::settings::EinkModeConfig::Dashboard {
                view: None,
                settle: chrono::TimeDelta::seconds(10),
                lead: chrono::TimeDelta::minutes(15),
            },
            orientation: crate::settings::Orientation::Portrait,
            refresh,
            sleep: None,
            partial: crate::settings::PartialRefresh {
                enabled: false,
                max_area_pct: 30,
                max_consecutive: 5,
            },
        }
    }

    fn display_with_feed(feed: RedditFeed) -> EinkDisplaySettings {
        EinkDisplaySettings {
            mode: crate::settings::EinkModeConfig::Reddit { feed },
            ..display_with_refresh(None)
        }
    }

    #[test]
    fn reddit_feed_falls_back_to_the_display() {
        let feed = RedditFeed {
            subreddit: "EarthPorn".to_owned(),
            timespan: RedditTimespan::Week,
            limit: 40,
        };

        assert_eq!(
            EpdFlagConfig::default().resolve_reddit_feed(&display_with_feed(feed.clone())),
            Some(feed)
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
            flag.resolve_reddit_feed(&display),
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
            EpdFlagConfig::default().resolve_reddit_feed(&display_with_refresh(None)),
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
            flag.resolve_reddit_feed(&display_with_refresh(None)),
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

        assert_eq!(flag.resolve_refresh_interval(Some(&display)), 5);
    }

    #[test]
    fn refresh_interval_falls_back_to_the_display() {
        let flag = EpdFlagConfig::default();

        assert_eq!(
            flag.resolve_refresh_interval(Some(&display_with_refresh(Some(
                chrono::TimeDelta::hours(1)
            )))),
            60
        );

        assert_eq!(
            flag.resolve_refresh_interval(Some(&display_with_refresh(Some(
                chrono::TimeDelta::seconds(30)
            )))),
            1
        );

        assert_eq!(
            flag.resolve_refresh_interval(Some(&display_with_refresh(None))),
            DEFAULT_REFRESH_INTERVAL_MINS
        );

        assert_eq!(
            flag.resolve_refresh_interval(None),
            DEFAULT_REFRESH_INTERVAL_MINS
        );
    }
}
