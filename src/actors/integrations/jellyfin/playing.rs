use crate::integrations::jellyfin::types::{Session, ticks_to_seconds};

#[derive(Debug, Clone, PartialEq)]
pub struct Playing {
    pub session_id: String,
    pub user: String,
    pub device: String,
    pub client: String,
    pub item_id: String,
    pub item_name: String,
    pub item_type: String,
    pub series_name: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub position_seconds: Option<f64>,
    pub runtime_seconds: Option<f64>,
    pub play_method: Option<String>,
    pub paused: bool,
}

impl Playing {
    pub fn from_session(session: &Session) -> Option<Self> {
        let item = session.now_playing_item.as_ref()?;
        let play_state = session.play_state.as_ref();

        Some(Self {
            session_id: session.id.clone(),
            user: session.user_name.clone().unwrap_or_default(),
            device: session.device_name.clone().unwrap_or_default(),
            client: session.client.clone().unwrap_or_default(),
            item_id: item.id.clone(),
            item_name: item.name.clone(),
            item_type: item.item_type.clone(),
            series_name: item.series_name.clone(),
            season: item.parent_index_number,
            episode: item.index_number,
            position_seconds: play_state
                .and_then(|p| p.position_ticks)
                .map(ticks_to_seconds),
            runtime_seconds: item.run_time_ticks.map(ticks_to_seconds),
            play_method: play_state.and_then(|p| p.play_method.clone()),
            paused: play_state.is_some_and(|p| p.is_paused),
        })
    }
}
