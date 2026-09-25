use ractor::Actor;
use uuid::Uuid;

use crate::{
    event_bus::{EventBusMessage, PlaybackState},
    integrations::jellyfin::{Jellyfin, types::Session},
    state::AppState,
};

pub mod edge;
pub mod message;
pub mod playing;
pub mod reconcile;

pub use message::JellyfinMessage;

use playing::Playing;
use reconcile::{Sessions, reconcile};

pub struct JellyfinActor {
    pub shared_actor_state: AppState,
    pub jellyfin: Jellyfin,
}

impl JellyfinActor {
    pub const NAME: &str = "jellyfin";

    async fn apply(&self, state: &mut Sessions, sessions: &[Session]) {
        for edge in reconcile(state, sessions) {
            let event_id = Uuid::new_v4();
            let playing = edge.playing;

            tracing::info!(
                "jellyfin {} {} on {} ({})",
                edge.state.as_str(),
                playing.item_name,
                playing.device,
                playing.user
            );

            if let Err(e) = self.save_event(event_id, edge.state, &playing).await {
                tracing::error!("failed to persist jellyfin playback event: {e}");
            }

            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Jellyfin {
                    event_id,
                    state: edge.state,
                    session_id: playing.session_id,
                    user: playing.user,
                    device: playing.device,
                    client: playing.client,
                    item_id: playing.item_id,
                    item_name: playing.item_name,
                    item_type: playing.item_type,
                    series_name: playing.series_name,
                    season: playing.season,
                    episode: playing.episode,
                    position_seconds: playing.position_seconds,
                    runtime_seconds: playing.runtime_seconds,
                    play_method: playing.play_method,
                });
        }

        for playing in state.values() {
            if let Err(e) = self
                .shared_actor_state
                .repos
                .jellyfin()
                .refresh_latest(playing)
                .await
            {
                tracing::error!("failed to refresh jellyfin session progress: {e}");
            }
        }
    }

    async fn save_event(
        &self,
        event_id: Uuid,
        playback: PlaybackState,
        playing: &Playing,
    ) -> Result<(), sqlx::Error> {
        let repo = self.shared_actor_state.repos.jellyfin();

        repo.append_event(event_id, playback, playing).await?;

        match playback {
            PlaybackState::Stopped => {
                repo.delete_session(&playing.user, &playing.session_id)
                    .await
            }
            PlaybackState::Started | PlaybackState::Paused | PlaybackState::Resumed => {
                repo.upsert_latest(event_id, playing).await
            }
        }
    }
}

impl Actor for JellyfinActor {
    type Msg = JellyfinMessage;
    type State = Sessions;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let poll_interval = self
            .shared_actor_state
            .settings
            .integrations
            .jellyfin
            .poll_interval
            .to_std()
            .map_err(|e| anyhow::anyhow!("invalid jellyfin poll_interval: {e}"))?;

        self.shared_actor_state
            .repos
            .jellyfin()
            .clear_sessions()
            .await?;

        myself.send_interval(poll_interval, || JellyfinMessage::Poll);
        myself.send_message(JellyfinMessage::Poll)?;

        Ok(Sessions::new())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let sessions = match message {
            JellyfinMessage::Snapshot(sessions) => {
                tracing::debug!("jellyfin pushed {} session(s)", sessions.len());
                sessions
            }
            JellyfinMessage::Poll => match self.jellyfin.sessions().await {
                Ok(sessions) => {
                    tracing::debug!("jellyfin poll returned {} session(s)", sessions.len());
                    sessions
                }
                Err(e) => {
                    tracing::warn!("failed to poll jellyfin sessions: {e}");
                    return Ok(());
                }
            },
        };

        self.apply(state, &sessions).await;

        Ok(())
    }
}
