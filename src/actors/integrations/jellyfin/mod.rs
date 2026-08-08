use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use ractor::{Actor, actor::actor_cell::ACTIVE_STATES};
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

/// `ForceKeepAlive` carries the timeout in seconds after which the server drops a
/// silent client; keep well inside it.
fn keep_alive_period(data: &Value) -> Option<Duration> {
    let timeout = data
        .as_u64()
        .or_else(|| data.as_str().and_then(|s| s.parse().ok()))?;

    (timeout > 1).then(|| Duration::from_secs(timeout / 2))
}

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

    const SESSIONS_INTERVAL_MS: u64 = 1500;
    const RECONNECT_MIN_DELAY: Duration = Duration::from_secs(1);
    const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(60);
    const STABLE_CONNECTION: Duration = Duration::from_secs(60);
    const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);
    const SILENCE_TIMEOUT: Duration = Duration::from_secs(60);

    /// Keep a socket up for as long as the actor lives, reconnecting with
    /// exponential backoff. Deliberately never stops the actor: the `/Sessions`
    /// poll is the fallback that keeps state fresh while the socket is down.
    async fn run(jellyfin: Jellyfin, myself: ractor::ActorRef<JellyfinMessage>) {
        let mut delay = Self::RECONNECT_MIN_DELAY;

        while ACTIVE_STATES.contains(&myself.get_status()) {
            let connected_at = Instant::now();

            let forward = |sessions| {
                myself
                    .send_message(JellyfinMessage::Snapshot(sessions))
                    .is_ok()
            };

            match Self::listen(&jellyfin, &forward).await {
                Ok(()) => return,
                Err(e) => tracing::warn!("jellyfin websocket disconnected: {e}"),
            }

            if connected_at.elapsed() >= Self::STABLE_CONNECTION {
                delay = Self::RECONNECT_MIN_DELAY;
            }

            tracing::info!("reconnecting to jellyfin in {delay:?}");
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Self::RECONNECT_MAX_DELAY);
        }
    }

    /// Hold one connection open, handing every session snapshot to `on_sessions`.
    /// Returns `Ok` only once `on_sessions` reports its consumer is gone, and `Err`
    /// on anything the caller should reconnect after.
    async fn listen(
        jellyfin: &Jellyfin,
        on_sessions: &impl Fn(Vec<Session>) -> bool,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("connecting to jellyfin websocket");
        let (socket, _) = tokio_tungstenite::connect_async(jellyfin.ws_url()).await?;
        let (mut write, mut read) = socket.split();

        write
            .send(WsMessage::text(
                json!({
                    "MessageType": "SessionsStart",
                    "Data": format!("0,{}", Self::SESSIONS_INTERVAL_MS),
                })
                .to_string(),
            ))
            .await?;

        let mut keep_alive = tokio::time::interval(Self::KEEP_ALIVE_INTERVAL);
        let mut saw_sessions = false;

        loop {
            let message = tokio::select! {
                _ = keep_alive.tick() => {
                    write.send(WsMessage::text(json!({ "MessageType": "KeepAlive" }).to_string())).await?;
                    continue;
                }
                message = tokio::time::timeout(Self::SILENCE_TIMEOUT, read.next()) => message,
            };

            let message = match message {
                Ok(Some(message)) => message?,
                Ok(None) => return Err(anyhow::anyhow!("jellyfin websocket stream ended")),
                Err(_) if saw_sessions => {
                    return Err(anyhow::anyhow!(
                        "no jellyfin message for {:?}, assuming the connection is dead",
                        Self::SILENCE_TIMEOUT
                    ));
                }
                Err(_) => {
                    tracing::warn!(
                        "jellyfin accepted the socket but has not pushed any sessions; \
                         relying on the /Sessions poll"
                    );
                    continue;
                }
            };

            let text = match message {
                WsMessage::Text(text) => text,
                WsMessage::Ping(payload) => {
                    write.send(WsMessage::Pong(payload)).await?;
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
                    write
                        .send(WsMessage::text(
                            json!({ "MessageType": "KeepAlive" }).to_string(),
                        ))
                        .await?;

                    if let Some(period) = keep_alive_period(&payload["Data"]) {
                        tracing::debug!("jellyfin asked for keepalives every {period:?}");
                        keep_alive = tokio::time::interval(period);
                    }
                }
                Some("Sessions") => match serde_json::from_value(payload["Data"].clone()) {
                    Ok(sessions) => {
                        saw_sessions = true;
                        if !on_sessions(sessions) {
                            return Ok(());
                        }
                    }
                    Err(e) => tracing::warn!("failed to parse jellyfin sessions payload: {e}"),
                },
                _ => {}
            }
        }
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
        tokio::spawn(Self::run(jellyfin, myself));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::JellyfinSettings;
    use tokio::net::TcpListener;

    const SESSIONS_FRAME: &str = r#"{"MessageType":"Sessions","Data":[{"Id":"s1","UserName":"anurag","Client":"Jellyfin Web","DeviceName":"Living Room TV","NowPlayingItem":{"Id":"m1","Name":"Arrival","Type":"Movie"},"PlayState":{"IsPaused":false,"PositionTicks":0}}]}"#;

    /// Accept one connection, push a `Sessions` frame, then hang up — the abrupt
    /// disconnect the reconnect loop has to survive.
    async fn push_then_hang_up(listener: TcpListener) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();

        let start = socket.next().await.unwrap().unwrap();
        assert!(start.to_text().unwrap().contains("SessionsStart"));

        socket.send(WsMessage::text(SESSIONS_FRAME)).await.unwrap();
        socket.close(None).await.unwrap();
    }

    async fn jellyfin_at(port: u16) -> Jellyfin {
        Jellyfin::new(&JellyfinSettings {
            url: format!("http://127.0.0.1:{port}"),
            api_key: Some("test-key".to_owned()),
            poll_interval: chrono::TimeDelta::seconds(30),
        })
        .expect("client")
    }

    #[tokio::test]
    async fn forwards_sessions_then_reports_the_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(push_then_hang_up(listener));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let client = jellyfin_at(port).await;

        let error = JellyfinActor::listen(&client, &|sessions| tx.send(sessions).is_ok())
            .await
            .expect_err("a server hang-up must surface as an error so the caller reconnects");
        assert!(error.to_string().contains("jellyfin"), "{error}");

        let sessions = rx.recv().await.expect("a session snapshot");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "s1");
        assert_eq!(
            sessions[0].now_playing_item.as_ref().unwrap().name,
            "Arrival"
        );
    }

    #[tokio::test]
    async fn stops_cleanly_once_the_consumer_is_gone() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(push_then_hang_up(listener));

        let client = jellyfin_at(port).await;

        JellyfinActor::listen(&client, &|_| false)
            .await
            .expect("a dead consumer ends the loop without asking for a reconnect");
    }

    #[tokio::test]
    async fn unreachable_server_is_an_error_not_a_hang() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let client = jellyfin_at(port).await;

        JellyfinActor::listen(&client, &|_| true)
            .await
            .expect_err("a refused connection must surface as an error");
    }

    #[test]
    fn keep_alive_period_is_half_the_server_timeout() {
        assert_eq!(keep_alive_period(&json!(60)), Some(Duration::from_secs(30)));
        assert_eq!(
            keep_alive_period(&json!("60")),
            Some(Duration::from_secs(30))
        );
        assert_eq!(keep_alive_period(&json!(1)), None);
        assert_eq!(keep_alive_period(&json!("nonsense")), None);
    }
}
