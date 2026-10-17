use cynic::{GraphQlResponse, SubscriptionBuilder};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL;

use super::query::{NotificationSubscription, TemperatureSubscription};
use super::types::{UnraidNotification, UnraidPush};
use super::{Unraid, response_data};
use crate::reconnect::{Attempt, Reconnect};
use crate::settings::UnraidWebsocketSettings;

const PROTOCOL: &str = "graphql-transport-ws";
const TEMPERATURE_ID: &str = "temperature";
const NOTIFICATION_ID: &str = "notification";

pub async fn stream(
    unraid: &Unraid,
    websocket: &UnraidWebsocketSettings,
    pushes: UnboundedSender<UnraidPush>,
) {
    let mut reconnect = Reconnect::new(websocket.reconnect);

    let forward = |push| {
        if let Err(e) = pushes.send(push) {
            tracing::error!("error forwarding an unraid push: {e}");
        }
    };

    loop {
        let result = listen(unraid, websocket, &mut reconnect, forward).await;
        let attempt = reconnect.failed();

        if let Err(e) = result {
            log_failure(&attempt, &e);
        }

        tokio::time::sleep(attempt.delay).await;
    }
}

fn log_failure(attempt: &Attempt, error: &anyhow::Error) {
    if !attempt.log {
        tracing::debug!(
            "unraid websocket error after {} attempts, reconnecting in {:?}: {error}",
            attempt.failures,
            attempt.delay
        );
        return;
    }

    tracing::warn!(
        "unraid websocket error, reconnecting in {:?}: {error}",
        attempt.delay
    );

    if attempt.last_log {
        tracing::warn!(
            "unraid websocket failed {} times, further errors logged at debug until it reconnects",
            attempt.failures
        );
    }
}

fn push(id: Option<&str>, payload: Value) -> Result<Option<UnraidPush>, anyhow::Error> {
    let push = match id {
        Some(TEMPERATURE_ID) => {
            let response: GraphQlResponse<TemperatureSubscription> =
                serde_json::from_value(payload)?;

            let readings = response_data(response)?
                .system_metrics_temperature
                .map(|metrics| metrics.disk_temperatures())
                .unwrap_or_default();

            UnraidPush::Temperatures(readings)
        }
        Some(NOTIFICATION_ID) => {
            let response: GraphQlResponse<NotificationSubscription> =
                serde_json::from_value(payload)?;

            UnraidPush::Notification(UnraidNotification::from(
                response_data(response)?.notification_added,
            ))
        }
        Some(other) => {
            tracing::debug!("ignoring a push for unknown unraid subscription {other}");
            return Ok(None);
        }
        None => {
            tracing::debug!("ignoring an unraid push without a subscription id");
            return Ok(None);
        }
    };

    Ok(Some(push))
}

fn subscribe(id: &str, payload: impl serde::Serialize) -> WsMessage {
    WsMessage::text(
        json!({
            "id": id,
            "type": "subscribe",
            "payload": payload,
        })
        .to_string(),
    )
}

async fn listen(
    unraid: &Unraid,
    websocket: &UnraidWebsocketSettings,
    reconnect: &mut Reconnect,
    on_push: impl Fn(UnraidPush),
) -> Result<(), anyhow::Error> {
    let silence_timeout = websocket.silence_timeout();

    tracing::info!("connecting to unraid websocket");
    let mut request = unraid.ws_url().into_client_request()?;

    request
        .headers_mut()
        .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static(PROTOCOL));

    let (socket, _) = tokio_tungstenite::connect_async(request).await?;
    let (mut write, mut read) = socket.split();

    write
        .send(WsMessage::text(
            json!({
                "type": "connection_init",
                "payload": { "x-api-key": unraid.api_key() },
            })
            .to_string(),
        ))
        .await?;

    let mut keep_alive = tokio::time::interval(websocket.keep_alive());
    let mut acknowledged = false;

    loop {
        let message = tokio::select! {
            _ = keep_alive.tick(), if acknowledged => {
                write.send(WsMessage::text(json!({ "type": "ping" }).to_string())).await?;
                continue;
            }
            message = tokio::time::timeout(silence_timeout, read.next()) => message,
        };

        let message = match message {
            Ok(Some(message)) => message?,
            Ok(None) => return Err(anyhow::anyhow!("unraid websocket stream ended")),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "no unraid message for {silence_timeout:?}, assuming the connection is dead"
                ));
            }
        };

        let text = match message {
            WsMessage::Text(text) => text,
            WsMessage::Ping(payload) => {
                write.send(WsMessage::Pong(payload)).await?;
                continue;
            }
            WsMessage::Close(frame) => {
                let reason = frame
                    .map(|frame| frame.reason.to_string())
                    .unwrap_or_default();

                return Err(anyhow::anyhow!("unraid closed the connection: {reason}"));
            }
            _ => continue,
        };

        let mut payload: Value = match serde_json::from_str(&text) {
            Ok(payload) => payload,
            Err(e) => {
                tracing::warn!("failed to parse unraid message: {e}");
                continue;
            }
        };

        match payload.get("type").and_then(Value::as_str) {
            Some("connection_ack") => {
                acknowledged = true;

                if reconnect.connected() {
                    tracing::info!("unraid websocket reconnected");
                }

                write
                    .send(subscribe(
                        TEMPERATURE_ID,
                        TemperatureSubscription::build(()),
                    ))
                    .await?;

                write
                    .send(subscribe(
                        NOTIFICATION_ID,
                        NotificationSubscription::build(()),
                    ))
                    .await?;

                on_push(UnraidPush::Subscribed);
            }
            Some("next") => {
                let id = payload.get("id").and_then(Value::as_str).map(str::to_owned);

                match push(id.as_deref(), payload["payload"].take()) {
                    Ok(Some(push)) => on_push(push),
                    Ok(None) => {}
                    Err(e) => tracing::warn!("failed to read an unraid push: {e}"),
                }
            }
            Some("ping") => {
                write
                    .send(WsMessage::text(json!({ "type": "pong" }).to_string()))
                    .await?;
            }
            Some("pong") => tracing::trace!("unraid answered a ping"),
            Some("error") => tracing::warn!(
                "unraid rejected the {} subscription: {}",
                payload["id"],
                payload["payload"]
            ),
            Some("complete") => {
                tracing::warn!("unraid completed the {} subscription", payload["id"]);
            }
            Some(other) => tracing::trace!("ignoring unraid {other} message"),
            None => tracing::debug!("unraid message without a type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use chrono::TimeDelta;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
    use tokio_tungstenite::tungstenite::http::StatusCode;

    use crate::integrations::unraid::types::{DiskTemperature, UnraidImportance};
    use crate::settings::enabled_state::EnabledState;
    use crate::settings::{BackoffSettings, ReconnectSettings, UnraidSettings};

    const TEMPERATURE_FRAME: &str = r#"{"id":"temperature","type":"next","payload":{"data":{"systemMetricsTemperature":{"sensors":[{"id":"disk:WD-WX72A70242TP","type":"DISK","current":{"value":30}}]}}}}"#;

    const NOTIFICATION_FRAME: &str = r#"{"id":"notification","type":"next","payload":{"data":{"notificationAdded":{"id":"n1","title":"Unraid Status","subject":"Warning [TOWER] - disk1 is hot","description":"52 C","importance":"WARNING","link":null}}}}"#;

    async fn push_then_hang_up(listener: TcpListener) {
        let (stream, _) = listener.accept().await.unwrap();

        #[allow(clippy::result_large_err)]
        let negotiate = |request: &Request, mut response: Response| {
            let protocol = request
                .headers()
                .get(SEC_WEBSOCKET_PROTOCOL)
                .and_then(|value| value.to_str().ok());

            if protocol != Some(PROTOCOL) {
                let mut denied = ErrorResponse::new(None);
                *denied.status_mut() = StatusCode::BAD_REQUEST;

                return Err(denied);
            }

            response
                .headers_mut()
                .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static(PROTOCOL));

            Ok(response)
        };

        let mut socket = tokio_tungstenite::accept_hdr_async(stream, negotiate)
            .await
            .unwrap();

        let init: Value =
            serde_json::from_str(socket.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();

        assert_eq!(init["type"], "connection_init");
        assert_eq!(init["payload"]["x-api-key"], "test-key");

        socket
            .send(WsMessage::text(r#"{"type":"connection_ack"}"#))
            .await
            .unwrap();

        for (id, field) in [
            (TEMPERATURE_ID, "systemMetricsTemperature"),
            (NOTIFICATION_ID, "notificationAdded"),
        ] {
            let subscription: Value =
                serde_json::from_str(socket.next().await.unwrap().unwrap().to_text().unwrap())
                    .unwrap();

            assert_eq!(subscription["type"], "subscribe");
            assert_eq!(subscription["id"], id);
            assert!(
                subscription["payload"]["query"]
                    .as_str()
                    .unwrap()
                    .contains(field)
            );
        }

        socket
            .send(WsMessage::text(TEMPERATURE_FRAME))
            .await
            .unwrap();
        socket
            .send(WsMessage::text(NOTIFICATION_FRAME))
            .await
            .unwrap();
        socket.close(None).await.unwrap();
    }

    fn websocket() -> UnraidWebsocketSettings {
        UnraidWebsocketSettings {
            keep_alive: TimeDelta::seconds(30),
            silence_timeout: TimeDelta::seconds(60),
            reconnect: ReconnectSettings {
                backoff: BackoffSettings {
                    min: TimeDelta::seconds(1),
                    max: TimeDelta::seconds(60),
                },
                log_attempts: 3,
            },
        }
    }

    fn unraid_at(port: u16) -> Unraid {
        Unraid::new(
            &UnraidSettings {
                state: EnabledState::Enabled,
                url: format!("http://127.0.0.1:{port}"),
                api_key: Some("test-key".to_owned()),
                refresh: TimeDelta::minutes(30),
                websocket: websocket(),
                notification_backfill: 50,
            },
            Duration::from_secs(30),
        )
        .expect("client")
    }

    #[tokio::test]
    async fn forwards_temperatures_and_notifications_then_reports_the_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(push_then_hang_up(listener));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut reconnect = Reconnect::new(websocket().reconnect);

        let error = listen(&unraid_at(port), &websocket(), &mut reconnect, |push| {
            tx.send(push).unwrap();
        })
        .await
        .expect_err("a server hang-up must surface as an error so the caller reconnects");

        assert!(error.to_string().contains("unraid"), "{error}");

        assert_eq!(rx.recv().await.expect("subscribed"), UnraidPush::Subscribed);

        assert_eq!(
            rx.recv().await.expect("temperatures"),
            UnraidPush::Temperatures(vec![DiskTemperature {
                serial: "WD-WX72A70242TP".to_owned(),
                celsius: 30,
            }])
        );

        assert_eq!(
            rx.recv().await.expect("a notification"),
            UnraidPush::Notification(UnraidNotification {
                id: "n1".to_owned(),
                title: "Unraid Status".to_owned(),
                subject: "Warning [TOWER] - disk1 is hot".to_owned(),
                description: "52 C".to_owned(),
                importance: UnraidImportance::Warning,
                link: None,
            })
        );
    }

    #[tokio::test]
    async fn unreachable_server_is_an_error_not_a_hang() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut reconnect = Reconnect::new(websocket().reconnect);

        listen(&unraid_at(port), &websocket(), &mut reconnect, |_| {})
            .await
            .expect_err("a refused connection must surface as an error");
    }

    #[test]
    fn the_websocket_url_follows_the_http_scheme() {
        assert_eq!(unraid_at(80).ws_url(), "ws://127.0.0.1:80/graphql");
    }
}
