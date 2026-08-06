use std::collections::HashMap;

use chrono::{NaiveTime, TimeDelta};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::timedelta_format::{option_time_delta_from_str, time_delta_from_str};

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
        Ok(SleepWindow {
            start: parse(&self.start)?,
            end: parse(&self.end)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SleepWindow {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl SleepWindow {
    pub fn contains(&self, now: NaiveTime) -> bool {
        if self.start <= self.end {
            now >= self.start && now < self.end
        } else {
            now >= self.start || now < self.end
        }
    }

    pub fn minutes_until_end(&self, now: NaiveTime) -> u32 {
        let mut delta = (self.end - now).num_minutes();
        if delta <= 0 {
            delta += 24 * 60;
        }
        delta as u32
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

        Ok(EinkDisplaySettings {
            name: self.name,
            firmware_version: self.firmware_version,
            mode: self.mode.resolve(),
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
