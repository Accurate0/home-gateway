use std::collections::{HashMap, HashSet};

use crate::event_bus::PlaybackState;
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
    fn from_session(session: &Session) -> Option<Self> {
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

pub type Sessions = HashMap<String, Playing>;

/// Diff a full session snapshot against the last known one and return the
/// playback edges it implies, updating `state` to the snapshot. Both the
/// WebSocket push and the `/Sessions` poll feed this, so a repeated identical
/// snapshot yields no edges. Progress within an item is not an edge — callers
/// still refresh the latest-state rows from `state` afterwards.
pub fn reconcile(state: &mut Sessions, sessions: &[Session]) -> Vec<(PlaybackState, Playing)> {
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    for session in sessions {
        let Some(playing) = Playing::from_session(session) else {
            continue;
        };
        seen.insert(playing.session_id.clone());

        match state.get(&playing.session_id) {
            None => edges.push((PlaybackState::Started, playing.clone())),
            Some(previous) if previous.item_id != playing.item_id => {
                edges.push((PlaybackState::Stopped, previous.clone()));
                edges.push((PlaybackState::Started, playing.clone()));
            }
            Some(previous) if previous.paused != playing.paused => {
                let state = if playing.paused {
                    PlaybackState::Paused
                } else {
                    PlaybackState::Resumed
                };
                edges.push((state, playing.clone()));
            }
            Some(_) => {}
        }

        state.insert(playing.session_id.clone(), playing);
    }

    let gone = state
        .keys()
        .filter(|id| !seen.contains(*id))
        .cloned()
        .collect::<Vec<_>>();

    for id in gone {
        if let Some(previous) = state.remove(&id) {
            edges.push((PlaybackState::Stopped, previous));
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::jellyfin::types::{Item, PlayState};

    fn session(item_id: Option<&str>, paused: bool, position: i64) -> Session {
        Session {
            id: "session-1".to_owned(),
            user_name: Some("anurag".to_owned()),
            client: Some("Jellyfin Web".to_owned()),
            device_name: Some("Living Room TV".to_owned()),
            now_playing_item: item_id.map(|id| Item {
                id: id.to_owned(),
                name: format!("item {id}"),
                item_type: "Movie".to_owned(),
                series_name: None,
                parent_index_number: None,
                index_number: None,
                run_time_ticks: Some(72_000_000_000),
            }),
            play_state: Some(PlayState {
                position_ticks: Some(position),
                is_paused: paused,
                play_method: Some("DirectPlay".to_owned()),
            }),
        }
    }

    fn states(edges: &[(PlaybackState, Playing)]) -> Vec<PlaybackState> {
        edges.iter().map(|(state, _)| *state).collect()
    }

    #[test]
    fn full_playback_lifecycle_emits_one_edge_each() {
        let mut state = Sessions::new();

        assert!(reconcile(&mut state, &[]).is_empty());

        let edges = reconcile(&mut state, &[session(Some("movie-a"), false, 0)]);
        assert_eq!(states(&edges), vec![PlaybackState::Started]);
        assert_eq!(edges[0].1.item_id, "movie-a");
        assert_eq!(edges[0].1.runtime_seconds, Some(7200.0));

        let edges = reconcile(&mut state, &[session(Some("movie-a"), true, 100)]);
        assert_eq!(states(&edges), vec![PlaybackState::Paused]);

        let edges = reconcile(&mut state, &[session(Some("movie-a"), false, 100)]);
        assert_eq!(states(&edges), vec![PlaybackState::Resumed]);

        let edges = reconcile(&mut state, &[session(Some("movie-b"), false, 0)]);
        assert_eq!(
            states(&edges),
            vec![PlaybackState::Stopped, PlaybackState::Started]
        );
        assert_eq!(edges[0].1.item_id, "movie-a");
        assert_eq!(edges[1].1.item_id, "movie-b");

        let edges = reconcile(&mut state, &[]);
        assert_eq!(states(&edges), vec![PlaybackState::Stopped]);
        assert!(state.is_empty());
    }

    #[test]
    fn repeated_snapshot_emits_nothing() {
        let mut state = Sessions::new();
        reconcile(&mut state, &[session(Some("movie-a"), false, 0)]);

        let snapshot = [session(Some("movie-a"), false, 0)];
        assert!(reconcile(&mut state, &snapshot).is_empty());
        assert!(reconcile(&mut state, &snapshot).is_empty());
    }

    #[test]
    fn progress_only_change_emits_nothing_but_updates_position() {
        let mut state = Sessions::new();
        reconcile(&mut state, &[session(Some("movie-a"), false, 0)]);

        let edges = reconcile(&mut state, &[session(Some("movie-a"), false, 600_000_000)]);
        assert!(edges.is_empty());
        assert_eq!(state["session-1"].position_seconds, Some(60.0));
    }

    #[test]
    fn session_losing_its_item_stops_playback() {
        let mut state = Sessions::new();
        reconcile(&mut state, &[session(Some("movie-a"), false, 0)]);

        let edges = reconcile(&mut state, &[session(None, false, 0)]);
        assert_eq!(states(&edges), vec![PlaybackState::Stopped]);
        assert_eq!(edges[0].1.item_id, "movie-a");
        assert!(state.is_empty());
    }
}
