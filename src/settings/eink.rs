use std::collections::HashMap;

use chrono::{NaiveTime, TimeDelta};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::timedelta_format::{humanize, option_time_delta_from_str, time_delta_from_str};

pub const DEFAULT_ALBUM_PREFIX: &str = "eink-display/album/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum EinkMode {
    Dashboard,
    Album,
    Reddit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub enum EinkModeConfig {
    Dashboard {
        view: Option<String>,
        settle: TimeDelta,
        lead: TimeDelta,
    },
    Album {
        album: Option<String>,
    },
    Reddit {
        feed: RedditFeed,
    },
}

impl EinkModeConfig {
    pub fn name(&self) -> EinkMode {
        match self {
            EinkModeConfig::Dashboard { .. } => EinkMode::Dashboard,
            EinkModeConfig::Album { .. } => EinkMode::Album,
            EinkModeConfig::Reddit { .. } => EinkMode::Reddit,
        }
    }

    pub fn view(&self) -> Option<&str> {
        match self {
            EinkModeConfig::Dashboard { view, .. } => view.as_deref(),
            EinkModeConfig::Album { .. } | EinkModeConfig::Reddit { .. } => None,
        }
    }

    pub fn settle(&self) -> Option<TimeDelta> {
        match self {
            EinkModeConfig::Dashboard { settle, .. } => Some(*settle),
            EinkModeConfig::Album { .. } | EinkModeConfig::Reddit { .. } => None,
        }
    }

    pub fn lead(&self) -> Option<TimeDelta> {
        match self {
            EinkModeConfig::Dashboard { lead, .. } => Some(*lead),
            EinkModeConfig::Album { .. } | EinkModeConfig::Reddit { .. } => None,
        }
    }

    pub fn album(&self) -> Option<&str> {
        match self {
            EinkModeConfig::Album { album } => album.as_deref(),
            EinkModeConfig::Dashboard { .. } | EinkModeConfig::Reddit { .. } => None,
        }
    }

    pub fn feed(&self) -> Option<&RedditFeed> {
        match self {
            EinkModeConfig::Reddit { feed } => Some(feed),
            EinkModeConfig::Dashboard { .. } | EinkModeConfig::Album { .. } => None,
        }
    }
}

impl Orientation {
    pub fn target_dims(self) -> (u32, u32) {
        match self {
            Orientation::Portrait => (1200, 1600),
            Orientation::Landscape => (1600, 1200),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Orientation::Portrait => "portrait",
            Orientation::Landscape => "landscape",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashboardView {
    pub name: String,
    pub query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub name: String,
    pub prefix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum RedditTimespan {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

impl RedditTimespan {
    pub fn as_str(self) -> &'static str {
        match self {
            RedditTimespan::Hour => "hour",
            RedditTimespan::Day => "day",
            RedditTimespan::Week => "week",
            RedditTimespan::Month => "month",
            RedditTimespan::Year => "year",
            RedditTimespan::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct RedditFeed {
    pub subreddit: String,
    pub timespan: RedditTimespan,
    pub limit: u32,
}

pub const PALETTE_COLORS: [(&str, f32, f32, f32, u8); 6] = [
    ("black", 0.0, 0.0, 0.0, 0),
    ("white", 255.0, 255.0, 255.0, 1),
    ("yellow", 255.0, 255.0, 0.0, 2),
    ("red", 255.0, 0.0, 0.0, 3),
    ("blue", 0.0, 0.0, 255.0, 5),
    ("green", 0.0, 255.0, 0.0, 6),
];

#[derive(Debug, Clone, Copy)]
pub struct PaletteColor {
    pub name: &'static str,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub index: u8,
}

fn default_palette() -> Vec<PaletteColor> {
    PALETTE_COLORS
        .iter()
        .map(|&(name, r, g, b, index)| PaletteColor {
            name,
            r,
            g,
            b,
            index,
        })
        .collect()
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct RawEinkGlobal {
    #[serde(default)]
    views: HashMap<String, RawDashboardView>,
    #[serde(default)]
    albums: HashMap<String, RawAlbum>,
    #[serde(default)]
    palette: HashMap<String, RawPaletteColor>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
struct RawPaletteColor {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
struct RawDashboardView {
    #[serde(default)]
    query: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
struct RawAlbum {
    #[serde(default)]
    prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EinkGlobalSettings {
    pub views: HashMap<String, DashboardView>,
    pub albums: HashMap<String, Album>,
    pub palette: Vec<PaletteColor>,
}

impl Default for EinkGlobalSettings {
    fn default() -> Self {
        Self {
            views: HashMap::new(),
            albums: HashMap::new(),
            palette: default_palette(),
        }
    }
}

impl RawEinkGlobal {
    pub fn resolve(self) -> EinkGlobalSettings {
        let views = self
            .views
            .into_iter()
            .map(|(name, v)| {
                (
                    name.clone(),
                    DashboardView {
                        name,
                        query: v.query,
                    },
                )
            })
            .collect();
        let albums = self
            .albums
            .into_iter()
            .map(|(name, a)| {
                let prefix = a
                    .prefix
                    .unwrap_or_else(|| format!("{DEFAULT_ALBUM_PREFIX}{name}/"));
                (name.clone(), Album { name, prefix })
            })
            .collect();
        let palette = PALETTE_COLORS
            .iter()
            .map(|&(name, dr, dg, db, index)| {
                let (r, g, b) = self
                    .palette
                    .get(name)
                    .map(|c| {
                        (
                            c.r.clamp(0.0, 255.0),
                            c.g.clamp(0.0, 255.0),
                            c.b.clamp(0.0, 255.0),
                        )
                    })
                    .unwrap_or((dr, dg, db));
                PaletteColor {
                    name,
                    r,
                    g,
                    b,
                    index,
                }
            })
            .collect();
        EinkGlobalSettings {
            views,
            albums,
            palette,
        }
    }
}

impl EinkGlobalSettings {
    pub fn view(&self, name: &str) -> Option<&DashboardView> {
        self.views.get(name)
    }

    pub fn album(&self, name: &str) -> Option<&Album> {
        self.albums.get(name)
    }

    pub fn default_album(&self) -> Album {
        Album {
            name: "default".to_owned(),
            prefix: DEFAULT_ALBUM_PREFIX.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum RawEinkMode {
    Dashboard {
        #[serde(default)]
        view: Option<String>,
        #[serde(with = "time_delta_from_str")]
        #[schemars(with = "String")]
        settle: TimeDelta,
        #[serde(with = "time_delta_from_str")]
        #[schemars(with = "String")]
        lead: TimeDelta,
    },
    Album {
        #[serde(default)]
        album: Option<String>,
    },
    Reddit {
        subreddit: String,
        timespan: RedditTimespan,
        limit: u32,
    },
}

impl RawEinkMode {
    fn resolve(self) -> EinkModeConfig {
        match self {
            RawEinkMode::Dashboard { view, settle, lead } => {
                EinkModeConfig::Dashboard { view, settle, lead }
            }
            RawEinkMode::Album { album } => EinkModeConfig::Album { album },
            RawEinkMode::Reddit {
                subreddit,
                timespan,
                limit,
            } => EinkModeConfig::Reddit {
                feed: RedditFeed {
                    subreddit,
                    timespan,
                    limit,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct RawPartialRefresh {
    enabled: bool,
    max_area_pct: u32,
    max_consecutive: i32,
}

impl RawPartialRefresh {
    fn resolve(self, id: &str) -> Result<PartialRefresh, String> {
        if self.max_area_pct == 0 || self.max_area_pct > 100 {
            return Err(format!(
                "eink display {id}: partial.max_area_pct must be between 1 and 100, got {}",
                self.max_area_pct
            ));
        }

        if self.max_consecutive < 1 {
            return Err(format!(
                "eink display {id}: partial.max_consecutive must be at least 1, got {}",
                self.max_consecutive
            ));
        }

        Ok(PartialRefresh {
            enabled: self.enabled,
            max_area_pct: self.max_area_pct,
            max_consecutive: self.max_consecutive,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PartialRefresh {
    pub enabled: bool,
    pub max_area_pct: u32,
    pub max_consecutive: i32,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawSleepWindow {
    start: String,
    end: String,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    grace: TimeDelta,
}

impl RawSleepWindow {
    fn resolve(self, id: &str) -> Result<SleepWindow, String> {
        let parse = |value: &str| {
            NaiveTime::parse_from_str(value, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
                .map_err(|_| {
                    format!("eink display {id}: invalid sleep time `{value}`, expected HH:MM")
                })
        };
        let window = SleepWindow {
            start: parse(&self.start)?,
            end: parse(&self.end)?,
            grace: self.grace,
        };

        if self.grace >= window.duration() {
            return Err(format!(
                "eink display {id}: sleep grace `{}` is not shorter than the {}-long window",
                humanize(self.grace),
                humanize(window.duration())
            ));
        }

        Ok(window)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SleepWindow {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub grace: TimeDelta,
}

impl SleepWindow {
    pub fn contains(&self, now: NaiveTime) -> bool {
        if self.start <= self.end {
            now >= self.start && now < self.end
        } else {
            now >= self.start || now < self.end
        }
    }

    pub fn duration(&self) -> TimeDelta {
        let mut delta = self.end - self.start;
        if delta <= TimeDelta::zero() {
            delta += TimeDelta::days(1);
        }
        delta
    }

    pub fn secs_until_end(&self, now: NaiveTime) -> u32 {
        let mut delta = (self.end - now).num_seconds();
        if delta <= 0 {
            delta += 24 * 60 * 60;
        }
        delta as u32
    }

    pub fn ending_within_grace(&self, now: NaiveTime) -> bool {
        i64::from(self.secs_until_end(now)) <= self.grace.num_seconds()
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawEinkDisplayBlock {
    name: String,
    firmware_version: String,
    mode: RawEinkMode,
    #[serde(default)]
    orientation: Option<Orientation>,
    #[serde(default, with = "option_time_delta_from_str")]
    #[schemars(with = "Option<String>")]
    refresh: Option<TimeDelta>,
    #[serde(default)]
    sleep: Option<RawSleepWindow>,
    partial: RawPartialRefresh,
}

impl RawEinkDisplayBlock {
    pub(crate) fn resolve(self, id: &str) -> Result<EinkDisplaySettings, String> {
        let sleep = self.sleep.map(|s| s.resolve(id)).transpose()?;
        let partial = self.partial.resolve(id)?;
        let mode = self.mode.resolve();

        if let (Some(sleep), Some(lead)) = (sleep, mode.lead())
            && sleep.grace >= lead
        {
            return Err(format!(
                "eink display {id}: sleep grace `{}` must be shorter than the dashboard lead `{}`, \
                 or a display waking inside the grace is served a pre-render that has not run yet",
                humanize(sleep.grace),
                humanize(lead)
            ));
        }

        Ok(EinkDisplaySettings {
            name: self.name,
            firmware_version: self.firmware_version,
            mode,
            orientation: self.orientation.unwrap_or(Orientation::Portrait),
            refresh: self.refresh,
            sleep,
            partial,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EinkDisplaySettings {
    pub name: String,
    pub firmware_version: String,
    pub mode: EinkModeConfig,
    pub orientation: Orientation,
    pub refresh: Option<TimeDelta>,
    pub sleep: Option<SleepWindow>,
    pub partial: PartialRefresh,
}

impl EinkDisplaySettings {
    pub fn target_dims(&self) -> (u32, u32) {
        self.orientation.target_dims()
    }

    pub fn orientation_str(&self) -> &'static str {
        self.orientation.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    fn window(start: (u32, u32), end: (u32, u32), grace: TimeDelta) -> SleepWindow {
        SleepWindow {
            start: NaiveTime::from_hms_opt(start.0, start.1, 0).unwrap(),
            end: NaiveTime::from_hms_opt(end.0, end.1, 0).unwrap(),
            grace,
        }
    }

    fn at(h: u32, m: u32, s: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, s).unwrap()
    }

    #[rstest]
    #[case(at(7, 59, 23), 37)]
    #[case(at(7, 57, 12), 168)]
    #[case(at(0, 0, 13), 28787)]
    #[case(at(23, 0, 5), 32395)]
    fn a_sub_minute_remainder_does_not_roll_forward_a_day(
        #[case] now: NaiveTime,
        #[case] expected: u32,
    ) {
        let sleep = window((23, 0), (8, 0), TimeDelta::minutes(5));

        assert_eq!(sleep.secs_until_end(now), expected);
    }

    #[test]
    fn a_window_that_has_ended_rolls_forward_to_the_next_one() {
        let sleep = window((23, 0), (8, 0), TimeDelta::minutes(5));

        assert_eq!(sleep.secs_until_end(at(8, 0, 0)), 24 * 60 * 60);
        assert_eq!(sleep.secs_until_end(at(9, 0, 0)), 23 * 60 * 60);
    }

    #[test]
    fn a_same_day_window_counts_down_to_its_end() {
        let sleep = window((3, 0), (8, 0), TimeDelta::minutes(5));

        assert_eq!(sleep.secs_until_end(at(3, 0, 0)), 5 * 60 * 60);
        assert_eq!(sleep.secs_until_end(at(7, 59, 23)), 37);
    }

    #[rstest]
    #[case(at(7, 55, 0), true)]
    #[case(at(7, 57, 12), true)]
    #[case(at(7, 59, 23), true)]
    #[case(at(7, 54, 59), false)]
    #[case(at(23, 30, 0), false)]
    fn the_grace_covers_only_the_end_of_the_window(#[case] now: NaiveTime, #[case] expected: bool) {
        let sleep = window((23, 0), (8, 0), TimeDelta::minutes(5));

        assert_eq!(sleep.ending_within_grace(now), expected);
    }

    #[test]
    fn a_zero_grace_covers_nothing_inside_the_window() {
        let sleep = window((23, 0), (8, 0), TimeDelta::zero());

        assert!(!sleep.ending_within_grace(at(7, 59, 59)));
        assert!(!sleep.ending_within_grace(at(23, 0, 1)));
    }

    #[test]
    fn a_missing_grace_is_rejected() {
        let raw = serde_yaml::from_str::<RawSleepWindow>("start: \"23:00\"\nend: \"08:00\"\n");

        assert!(raw.is_err());
    }

    #[test]
    fn a_grace_that_swallows_the_window_is_rejected() {
        let raw = serde_yaml::from_str::<RawSleepWindow>(
            "start: \"23:00\"\nend: \"08:00\"\ngrace: 10h\n",
        )
        .unwrap();

        assert!(raw.resolve("hallway-epd").is_err());
    }

    #[test]
    fn a_grace_is_parsed_from_a_duration_string() {
        let raw =
            serde_yaml::from_str::<RawSleepWindow>("start: \"23:00\"\nend: \"08:00\"\ngrace: 5m\n")
                .unwrap();
        let sleep = raw.resolve("hallway-epd").unwrap();

        assert_eq!(sleep.grace, TimeDelta::minutes(5));
        assert_eq!(sleep.start, at(23, 0, 0));
        assert_eq!(sleep.end, at(8, 0, 0));
    }

    fn display_block(mode: &str, grace: Option<&str>) -> RawEinkDisplayBlock {
        let sleep = grace.map_or_else(String::new, |grace| {
            format!("sleep:\n  start: \"23:00\"\n  end: \"08:00\"\n  grace: {grace}\n")
        });
        let yaml = format!(
            "name: Hallway Display\nfirmware_version: v0.1.0\nrefresh: 1h\n{mode}{sleep}\
             partial:\n  enabled: true\n  max_area_pct: 45\n  max_consecutive: 12\n"
        );

        serde_yaml::from_str(&yaml).unwrap()
    }

    fn dashboard_mode(lead: &str) -> String {
        format!("mode:\n  name: dashboard\n  view: home\n  settle: 10s\n  lead: {lead}\n")
    }

    #[rstest]
    #[case("10m", "15m", true)]
    #[case("14m59s", "15m", true)]
    #[case("15m", "15m", false)]
    #[case("20m", "15m", false)]
    fn a_grace_must_stay_inside_the_dashboard_lead(
        #[case] grace: &str,
        #[case] lead: &str,
        #[case] accepted: bool,
    ) {
        let block = display_block(&dashboard_mode(lead), Some(grace));

        assert_eq!(block.resolve("hallway-epd").is_ok(), accepted);
    }

    #[test]
    fn a_lead_is_only_compared_when_a_window_is_configured() {
        let block = display_block(&dashboard_mode("15m"), None);

        assert!(block.resolve("hallway-epd").is_ok());
    }

    #[test]
    fn a_mode_without_a_lead_skips_the_comparison() {
        let mode = "mode:\n  name: album\n  album: landscapes\n".to_owned();
        let block = display_block(&mode, Some("20m"));

        assert!(block.resolve("hallway-epd").is_ok());
    }
}
