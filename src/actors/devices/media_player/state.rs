use serde::Deserialize;

use crate::event_bus::PlaybackState;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Attributes {
    #[serde(default)]
    pub media_title: Option<String>,
    #[serde(default)]
    pub media_artist: Option<String>,
    #[serde(default)]
    pub media_album_name: Option<String>,
    #[serde(default)]
    pub media_series_title: Option<String>,
    #[serde(default, deserialize_with = "crate::serde_lenient::opt_i32")]
    pub media_season: Option<i32>,
    #[serde(default, deserialize_with = "crate::serde_lenient::opt_i32")]
    pub media_episode: Option<i32>,
    #[serde(default)]
    pub media_content_type: Option<String>,
    #[serde(default, deserialize_with = "crate::serde_lenient::opt_f64")]
    pub media_duration: Option<f64>,
    #[serde(default, deserialize_with = "crate::serde_lenient::opt_f64")]
    pub media_position: Option<f64>,
    #[serde(default)]
    pub media_position_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, deserialize_with = "crate::serde_lenient::opt_f64")]
    pub volume_level: Option<f64>,
    #[serde(default)]
    pub is_volume_muted: Option<bool>,
    #[serde(default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub entity_picture: Option<String>,
}

/// What we remember about a player between updates, so a `state_changed` event
/// can be turned into a playback edge rather than a bare state string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prior {
    pub state: String,
    pub media_title: Option<String>,
}

/// Whether a raw Home Assistant `media_player` state counts as having something
/// loaded. Anything else (`idle`, `off`, `standby`, `unavailable`, `unknown`) is
/// treated as nothing playing.
fn is_active(state: &str) -> bool {
    matches!(state, "playing" | "paused" | "buffering")
}

fn is_paused(state: &str) -> bool {
    state == "paused"
}

/// Derive the playback edges a transition represents. Returns an empty vector
/// when nothing meaningful changed — a position-only update must not put an
/// event on the bus, mirroring the Jellyfin reconciler.
pub fn edges(prior: Option<&Prior>, now: &Prior) -> Vec<PlaybackState> {
    let was_active = prior.is_some_and(|p| is_active(&p.state));

    if !is_active(&now.state) {
        return if was_active {
            vec![PlaybackState::Stopped]
        } else {
            Vec::new()
        };
    }

    let Some(prior) = prior.filter(|_| was_active) else {
        return vec![PlaybackState::Started];
    };

    if prior.media_title != now.media_title {
        return vec![PlaybackState::Stopped, PlaybackState::Started];
    }

    if is_paused(&prior.state) == is_paused(&now.state) {
        return Vec::new();
    }

    if is_paused(&now.state) {
        vec![PlaybackState::Paused]
    } else {
        vec![PlaybackState::Resumed]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prior(state: &str, title: Option<&str>) -> Prior {
        Prior {
            state: state.to_owned(),
            media_title: title.map(str::to_owned),
        }
    }

    #[test]
    fn first_sight_of_a_playing_player_starts() {
        assert_eq!(
            edges(None, &prior("playing", Some("Andor"))),
            vec![PlaybackState::Started]
        );
    }

    #[test]
    fn first_sight_of_an_idle_player_is_not_an_edge() {
        assert!(edges(None, &prior("idle", None)).is_empty());
        assert!(edges(None, &prior("off", None)).is_empty());
    }

    #[test]
    fn going_idle_while_playing_stops() {
        assert_eq!(
            edges(Some(&prior("playing", Some("Andor"))), &prior("off", None)),
            vec![PlaybackState::Stopped]
        );
    }

    #[test]
    fn changing_item_stops_then_starts() {
        assert_eq!(
            edges(
                Some(&prior("playing", Some("Andor"))),
                &prior("playing", Some("Severance"))
            ),
            vec![PlaybackState::Stopped, PlaybackState::Started]
        );
    }

    #[test]
    fn pause_and_resume_are_single_edges() {
        assert_eq!(
            edges(
                Some(&prior("playing", Some("Andor"))),
                &prior("paused", Some("Andor"))
            ),
            vec![PlaybackState::Paused]
        );
        assert_eq!(
            edges(
                Some(&prior("paused", Some("Andor"))),
                &prior("playing", Some("Andor"))
            ),
            vec![PlaybackState::Resumed]
        );
    }

    #[test]
    fn position_only_updates_are_not_edges() {
        assert!(
            edges(
                Some(&prior("playing", Some("Andor"))),
                &prior("playing", Some("Andor"))
            )
            .is_empty()
        );
    }

    #[test]
    fn buffering_counts_as_playing() {
        assert!(
            edges(
                Some(&prior("playing", Some("Andor"))),
                &prior("buffering", Some("Andor"))
            )
            .is_empty()
        );
    }
}
