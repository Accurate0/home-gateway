use serde::Deserialize;

use super::{Item, PlayState};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub now_playing_item: Option<Item>,
    #[serde(default)]
    pub play_state: Option<PlayState>,
}
