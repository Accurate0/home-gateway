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

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct RawEinkGlobal {
    #[serde(default)]
    views: HashMap<String, RawDashboardView>,
    #[serde(default)]
    albums: HashMap<String, RawAlbum>,
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

#[derive(Debug, Clone, Default)]
pub struct EinkGlobalSettings {
    pub views: HashMap<String, DashboardView>,
    pub albums: HashMap<String, Album>,
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
        EinkGlobalSettings { views, albums }
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
