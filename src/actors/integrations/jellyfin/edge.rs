use super::playing::Playing;
use crate::media_control::PlaybackState;

pub struct Edge {
    pub state: PlaybackState,
    pub playing: Playing,
}
