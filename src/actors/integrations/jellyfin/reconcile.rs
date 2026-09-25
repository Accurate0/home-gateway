use std::collections::{HashMap, HashSet};

use crate::event_bus::PlaybackState;
use crate::integrations::jellyfin::types::Session;

use super::edge::Edge;
use super::playing::Playing;

pub type Sessions = HashMap<String, Playing>;

pub fn reconcile(state: &mut Sessions, sessions: &[Session]) -> Vec<Edge> {
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    for session in sessions {
        let Some(playing) = Playing::from_session(session) else {
            continue;
        };

        seen.insert(playing.session_id.clone());

        match state.get(&playing.session_id) {
            None => edges.push(Edge {
                state: PlaybackState::Started,
                playing: playing.clone(),
            }),
            Some(previous) if previous.item_id != playing.item_id => {
                edges.push(Edge {
                    state: PlaybackState::Stopped,
                    playing: previous.clone(),
                });
                edges.push(Edge {
                    state: PlaybackState::Started,
                    playing: playing.clone(),
                });
            }
            Some(previous) if previous.paused != playing.paused => {
                let state = if playing.paused {
                    PlaybackState::Paused
                } else {
                    PlaybackState::Resumed
                };

                edges.push(Edge {
                    state,
                    playing: playing.clone(),
                });
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
            edges.push(Edge {
                state: PlaybackState::Stopped,
                playing: previous,
            });
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

    fn states(edges: &[Edge]) -> Vec<PlaybackState> {
        edges.iter().map(|edge| edge.state).collect()
    }

    #[test]
    fn full_playback_lifecycle_emits_one_edge_each() {
        let mut state = Sessions::new();

        assert!(reconcile(&mut state, &[]).is_empty());

        let edges = reconcile(&mut state, &[session(Some("movie-a"), false, 0)]);

        assert_eq!(states(&edges), vec![PlaybackState::Started]);
        assert_eq!(edges[0].playing.item_id, "movie-a");
        assert_eq!(edges[0].playing.runtime_seconds, Some(7200.0));

        let edges = reconcile(&mut state, &[session(Some("movie-a"), true, 100)]);

        assert_eq!(states(&edges), vec![PlaybackState::Paused]);

        let edges = reconcile(&mut state, &[session(Some("movie-a"), false, 100)]);

        assert_eq!(states(&edges), vec![PlaybackState::Resumed]);

        let edges = reconcile(&mut state, &[session(Some("movie-b"), false, 0)]);

        assert_eq!(
            states(&edges),
            vec![PlaybackState::Stopped, PlaybackState::Started]
        );
        assert_eq!(edges[0].playing.item_id, "movie-a");
        assert_eq!(edges[1].playing.item_id, "movie-b");

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
        assert_eq!(edges[0].playing.item_id, "movie-a");
        assert!(state.is_empty());
    }
}
