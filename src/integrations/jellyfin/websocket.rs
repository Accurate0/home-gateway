use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ractor::ActorRef;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;

use super::Jellyfin;
use super::types::Session;
use crate::actors::integrations::jellyfin::{JellyfinActor, JellyfinMessage};
use crate::reconnect::{Attempt, Reconnect};
use crate::settings::JellyfinWebsocketSettings;

fn keep_alive_period(data: &Value) -> Option<Duration> {
    let timeout = data
        .as_u64()
        .or_else(|| data.as_str().and_then(|s| s.parse().ok()))?;

    (timeout > 1).then(|| Duration::from_secs(timeout / 2))
}

fn jellyfin_actor() -> Option<ActorRef<JellyfinMessage>> {
    ractor::registry::where_is(JellyfinActor::NAME).map(ActorRef::from)
}

fn forward(sessions: Vec<Session>) {
    let Some(actor) = jellyfin_actor() else {
        tracing::error!("jellyfin actor is not registered, dropping a session snapshot");
        return;
    };

    if let Err(e) = actor.send_message(JellyfinMessage::Snapshot(sessions)) {
        tracing::error!("error sending to jellyfin actor: {e}");
    }
}

pub async fn process_events(
    jellyfin: Jellyfin,
    websocket: JellyfinWebsocketSettings,
    cancellation_token: CancellationToken,
) {
    let mut reconnect = Reconnect::new(websocket.reconnect);

    loop {
        tokio::select! {
            result = listen(&jellyfin, &websocket, &mut reconnect, forward) => {
                let attempt = reconnect.failed();

                if let Err(e) = result {
                    log_failure(&attempt, &e);
                }

                tokio::time::sleep(attempt.delay).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("jellyfin websocket cancellation requested");
                return;
            }
        }
    }
}

fn log_failure(attempt: &Attempt, error: &anyhow::Error) {
    if !attempt.log {
        tracing::debug!(
            "jellyfin websocket error after {} attempts, reconnecting in {:?}: {error}",
            attempt.failures,
            attempt.delay
        );
        return;
    }

    tracing::warn!(
        "jellyfin websocket error, reconnecting in {:?}: {error}",
        attempt.delay
    );

    if attempt.last_log {
        tracing::warn!(
            "jellyfin websocket failed {} times, further errors logged at debug until it reconnects",
            attempt.failures
        );
    }
}

async fn listen(
    jellyfin: &Jellyfin,
    websocket: &JellyfinWebsocketSettings,
    reconnect: &mut Reconnect,
    on_sessions: impl Fn(Vec<Session>),
) -> Result<(), anyhow::Error> {
    let silence_timeout = websocket.silence_timeout();

    tracing::info!("connecting to jellyfin websocket");
    let (socket, _) = tokio_tungstenite::connect_async(jellyfin.ws_url()).await?;
    let (mut write, mut read) = socket.split();

    if reconnect.connected() {
        tracing::info!("jellyfin websocket reconnected");
    }

    write
        .send(WsMessage::text(
            json!({
                "MessageType": "SessionsStart",
                "Data": format!("0,{}", websocket.sessions_interval.num_milliseconds()),
            })
            .to_string(),
        ))
        .await?;

    let mut keep_alive = tokio::time::interval(websocket.keep_alive());
    let mut saw_sessions = false;

    loop {
        let message = tokio::select! {
            _ = keep_alive.tick() => {
                write.send(WsMessage::text(json!({ "MessageType": "KeepAlive" }).to_string())).await?;
                continue;
            }
            message = tokio::time::timeout(silence_timeout, read.next()) => message,
        };

        let message = match message {
            Ok(Some(message)) => message?,
            Ok(None) => return Err(anyhow::anyhow!("jellyfin websocket stream ended")),
            Err(_) if saw_sessions => {
                return Err(anyhow::anyhow!(
                    "no jellyfin message for {silence_timeout:?}, assuming the connection is dead"
                ));
            }
            Err(_) => {
                tracing::warn!(
                    "jellyfin accepted the socket but has not pushed any sessions, relying on the /Sessions poll"
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
                    on_sessions(sessions);
                }
                Err(e) => tracing::warn!("failed to parse jellyfin sessions payload: {e}"),
            },
            Some(other) => tracing::trace!("ignoring jellyfin {other} message"),
            None => tracing::debug!("jellyfin message without a MessageType"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeDelta;
    use tokio::net::TcpListener;

    use crate::settings::enabled_state::EnabledState;
    use crate::settings::{BackoffSettings, JellyfinSettings, ReconnectSettings};

    const SESSIONS_FRAME: &str = r#"{"MessageType":"Sessions","Data":[{"Id":"s1","UserName":"anurag","Client":"Jellyfin Web","DeviceName":"Living Room TV","NowPlayingItem":{"Id":"m1","Name":"Arrival","Type":"Movie"},"PlayState":{"IsPaused":false,"PositionTicks":0}}]}"#;

    async fn push_then_hang_up(listener: TcpListener) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();

        let start = socket.next().await.unwrap().unwrap();
        assert!(start.to_text().unwrap().contains("SessionsStart"));

        socket.send(WsMessage::text(SESSIONS_FRAME)).await.unwrap();
        socket.close(None).await.unwrap();
    }

    fn websocket() -> JellyfinWebsocketSettings {
        JellyfinWebsocketSettings {
            keep_alive: TimeDelta::seconds(30),
            silence_timeout: TimeDelta::seconds(60),
            sessions_interval: TimeDelta::milliseconds(1500),
            reconnect: ReconnectSettings {
                backoff: BackoffSettings {
                    min: TimeDelta::seconds(1),
                    max: TimeDelta::seconds(60),
                },
                log_attempts: 3,
            },
        }
    }

    fn jellyfin_at(port: u16) -> Jellyfin {
        Jellyfin::new(
            &JellyfinSettings {
                state: EnabledState::Enabled,
                url: format!("http://127.0.0.1:{port}"),
                api_key: Some("test-key".to_owned()),
                poll_interval: TimeDelta::seconds(30),
                websocket: websocket(),
            },
            Duration::from_secs(30),
        )
        .expect("client")
    }

    #[tokio::test]
    async fn forwards_sessions_then_reports_the_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(push_then_hang_up(listener));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let client = jellyfin_at(port);
        let mut reconnect = Reconnect::new(websocket().reconnect);

        let error = listen(&client, &websocket(), &mut reconnect, |sessions| {
            tx.send(sessions).unwrap();
        })
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
    async fn unreachable_server_is_an_error_not_a_hang() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let client = jellyfin_at(port);
        let mut reconnect = Reconnect::new(websocket().reconnect);

        listen(&client, &websocket(), &mut reconnect, |_| {})
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
