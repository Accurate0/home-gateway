use crate::event_bus::PlaybackState;

use super::playing::Playing;

pub struct Edge {
    pub state: PlaybackState,
    pub playing: Playing,
}
