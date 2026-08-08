use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ractor::Actor;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

use crate::{
    event_bus::{EventBusMessage, PlaybackState},
    integrations::jellyfin::{Jellyfin, types::Session},
    state::SharedActorState,
};

pub mod reconcile;

use reconcile::{Playing, Sessions, reconcile};

pub enum JellyfinMessage {
    Poll,
    Snapshot(Vec<Session>),
}

pub struct JellyfinActor {
    pub shared_actor_state: SharedActorState,
    pub jellyfin: Jellyfin,
}

impl JellyfinActor {
    pub const NAME: &str = "jellyfin";

    const RECONNECT_DELAY: Duration = Duration::from_secs(5);

    async fn listen(
        jellyfin: &Jellyfin,
        myself: &ractor::ActorRef<JellyfinMessage>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("connecting to jellyfin websocket");
        let (mut socket, _) = tokio_tungstenite::connect_async(jellyfin.ws_url()).await?;

        socket
            .send(WsMessage::text(
                json!({ "MessageType": "SessionsStart", "Data": "0,1500" }).to_string(),
            ))
            .await?;

        while let Some(message) = socket.next().await {
            let text = match message? {
                WsMessage::Text(text) => text,
                WsMessage::Ping(payload) => {
                    socket.send(WsMessage::Pong(payload)).await?;
                    continue;
                }
                WsMessage::Close(_) => {
                    return Err(anyhow::anyhow!("jellyfin closed the connection"));
                }
                _ => continue,
            };

            let payload: Value = match serde_json::from_str(&text) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::warn!("failed to parse jellyfin message: {e}");
                    continue;
                }
            };

            match payload.get("MessageType").and_then(Value::as_str) {
                Some("ForceKeepAlive") => {
                    socket
                        .send(WsMessage::text(
                            json!({ "MessageType": "KeepAlive" }).to_string(),
                        ))
                        .await?;
                }
                Some("Sessions") => match serde_json::from_value(payload["Data"].clone()) {
                    Ok(sessions) => myself.send_message(JellyfinMessage::Snapshot(sessions))?,
                    Err(e) => tracing::warn!("failed to parse jellyfin sessions payload: {e}"),
                },
                _ => {}
            }
        }

        Err(anyhow::anyhow!("jellyfin websocket stream ended"))
    }

    async fn apply(&self, state: &mut Sessions, sessions: &[Session]) {
        for (playback, playing) in reconcile(state, sessions) {
            let event_id = Uuid::new_v4();

            tracing::info!(
                "jellyfin {} {} on {} ({})",
                playback.as_str(),
                playing.item_name,
                playing.device,
                playing.user
            );

            if let Err(e) = self.save_event(event_id, playback, &playing).await {
                tracing::error!("failed to persist jellyfin playback event: {e}");
            }

            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Jellyfin {
                    event_id,
                    state: playback,
                    session_id: playing.session_id.clone(),
                    user: playing.user.clone(),
                    device: playing.device.clone(),
                    client: playing.client.clone(),
                    item_id: playing.item_id.clone(),
                    item_name: playing.item_name.clone(),
                    item_type: playing.item_type.clone(),
                    series_name: playing.series_name.clone(),
                    season: playing.season,
                    episode: playing.episode,
                    position_seconds: playing.position_seconds,
                    runtime_seconds: playing.runtime_seconds,
                    play_method: playing.play_method.clone(),
                });
        }

        for playing in state.values() {
            if let Err(e) = self.refresh_latest(playing).await {
                tracing::error!("failed to refresh jellyfin session progress: {e}");
            }
        }
    }

    async fn save_event(
        &self,
        event_id: Uuid,
        playback: PlaybackState,
        playing: &Playing,
    ) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"INSERT INTO jellyfin_playback_events (
                event_id, state, session_id, user_name, device_name, client,
                item_id, item_name, item_type, series_name, season, episode,
                position_seconds, runtime_seconds, play_method
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
            event_id,
            playback.as_str(),
            playing.session_id,
            playing.user,
            playing.device,
            playing.client,
            playing.item_id,
            playing.item_name,
            playing.item_type,
            playing.series_name,
            playing.season,
            playing.episode,
            playing.position_seconds,
            playing.runtime_seconds,
            playing.play_method,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        if playback != PlaybackState::Stopped {
            return self.save_latest(event_id, playing).await;
        }

        sqlx::query!(
            r#"UPDATE latest_jellyfin_session
            SET item_id = NULL,
                item_name = NULL,
                item_type = NULL,
                series_name = NULL,
                season = NULL,
                episode = NULL,
                position_seconds = NULL,
                play_method = NULL,
                paused = false,
                event_id = $2,
                updated_at = now()
            WHERE session_id = $1"#,
            playing.session_id,
            event_id,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn save_latest(&self, event_id: Uuid, playing: &Playing) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"INSERT INTO latest_jellyfin_session (
                session_id, user_name, device_name, client, item_id, item_name,
                item_type, series_name, season, episode, position_seconds,
                runtime_seconds, play_method, paused, event_id, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (session_id)
            DO UPDATE SET
                user_name = EXCLUDED.user_name,
                device_name = EXCLUDED.device_name,
                client = EXCLUDED.client,
                item_id = EXCLUDED.item_id,
                item_name = EXCLUDED.item_name,
                item_type = EXCLUDED.item_type,
                series_name = EXCLUDED.series_name,
                season = EXCLUDED.season,
                episode = EXCLUDED.episode,
                position_seconds = EXCLUDED.position_seconds,
                runtime_seconds = EXCLUDED.runtime_seconds,
                play_method = EXCLUDED.play_method,
                paused = EXCLUDED.paused,
                event_id = EXCLUDED.event_id,
                updated_at = EXCLUDED.updated_at
            "#,
            playing.session_id,
            playing.user,
            playing.device,
            playing.client,
            playing.item_id,
            playing.item_name,
            playing.item_type,
            playing.series_name,
            playing.season,
            playing.episode,
            playing.position_seconds,
            playing.runtime_seconds,
            playing.play_method,
            playing.paused,
            event_id,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn refresh_latest(&self, playing: &Playing) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"UPDATE latest_jellyfin_session
            SET position_seconds = $2, paused = $3, updated_at = now()
            WHERE session_id = $1"#,
            playing.session_id,
            playing.position_seconds,
            playing.paused,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
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
        let Some(settings) = self.shared_actor_state.settings.jellyfin.as_ref() else {
            return Err(anyhow::anyhow!("jellyfin is not configured").into());
        };

        let poll_interval = settings
            .poll_interval
            .to_std()
            .map_err(|e| anyhow::anyhow!("invalid jellyfin poll_interval: {e}"))?;

        myself.send_interval(poll_interval, || JellyfinMessage::Poll);
        myself.send_message(JellyfinMessage::Poll)?;

        let jellyfin = self.jellyfin.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::listen(&jellyfin, &myself).await {
                tracing::error!("jellyfin websocket error: {e}");
            }
            tokio::time::sleep(Self::RECONNECT_DELAY).await;
            myself.stop(Some("jellyfin websocket disconnected".to_owned()));
        });

        Ok(Sessions::new())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let sessions = match message {
            JellyfinMessage::Snapshot(sessions) => sessions,
            JellyfinMessage::Poll => match self.jellyfin.sessions().await {
                Ok(sessions) => sessions,
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
