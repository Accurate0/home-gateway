use serde::Deserialize;

const TICKS_PER_SECOND: f64 = 10_000_000.0;

pub fn ticks_to_seconds(ticks: i64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    #[serde(default)]
    pub position_ticks: Option<i64>,
    #[serde(default)]
    pub is_paused: bool,
    #[serde(default)]
    pub play_method: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "Type", default)]
    pub item_type: String,
    #[serde(default)]
    pub series_name: Option<String>,
    #[serde(default)]
    pub parent_index_number: Option<i32>,
    #[serde(default)]
    pub index_number: Option<i32>,
    #[serde(default)]
    pub run_time_ticks: Option<i64>,
}
