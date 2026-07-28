use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

pub const DEFAULT_ALBUM_PREFIX: &str = "eink-display/album/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum EinkMode {
    Dashboard,
    Album,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub enum EinkModeConfig {
    Dashboard { view: Option<String> },
    Album { album: Option<String> },
}

impl Default for EinkModeConfig {
    fn default() -> Self {
        EinkModeConfig::Dashboard { view: None }
    }
}

impl EinkModeConfig {
    pub fn name(&self) -> EinkMode {
        match self {
            EinkModeConfig::Dashboard { .. } => EinkMode::Dashboard,
            EinkModeConfig::Album { .. } => EinkMode::Album,
        }
    }

    pub fn view(&self) -> Option<&str> {
        match self {
            EinkModeConfig::Dashboard { view } => view.as_deref(),
            EinkModeConfig::Album { .. } => None,
        }
    }

    pub fn album(&self) -> Option<&str> {
        match self {
            EinkModeConfig::Album { album } => album.as_deref(),
            EinkModeConfig::Dashboard { .. } => None,
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
