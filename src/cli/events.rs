use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use super::client::Client;

const PROTOCOL: &str = "graphql-transport-ws";

const EVENTS_SUBSCRIPTION: &str = r#"
subscription($filter: String!) {
  events(filter: $filter) {
    __typename
    ... on PresenceUpdate { id present }
    ... on DoorUpdate { id open }
    ... on LightUpdate { id on brightness colourTemperature }
    ... on EnvironmentUpdate { id readings { metric value } }
    ... on HomeAssistantUpdate { entityId state }
    ... on UnifiUpdate { client connected }
    ... on WoolworthsUpdate { name oldPrice newPrice }
    ... on JellyfinUpdate { user state itemName seriesName }
    ... on WeatherUpdate { source readings { metric day value } }
    ... on FuelWatchUpdate { name change oldPrice newPrice }
    ... on SwitchUpdate { device action }
    ... on ModeUpdate { mode previous }
    ... on CronUpdate { name }
    ... on SunUpdate { transition }
    ... on DeviceBatteryUpdate { id kind batteryVoltage }
    ... on CommandFailedUpdate { id kind attempts }
    ... on CustomUpdate { source name payload }
    ... on MediaPlayerUpdate { deviceId state }
    ... on SolarUpdate { currentWh }
  }
}
"#;

pub fn websocket_url(base_url: &str, path: &str) -> String {
    let url = if let Some(rest) = base_url.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base_url.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        base_url.to_owned()
    };

    format!("{}{path}", url.trim_end_matches('/'))
}

pub async fn tail(client: &Client, filter: &str, as_json: bool) -> Result<()> {
    let mut request = websocket_url(client.base_url(), "/v1/graphql/ws")
        .into_client_request()
        .context("invalid websocket url")?;

    request
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", HeaderValue::from_static(PROTOCOL));

    let (mut socket, _) = connect_async(request)
        .await
        .context("failed to connect to the event stream")?;

    send(
        &mut socket,
        json!({ "type": "connection_init", "payload": client.auth_payload() }),
    )
    .await?;

    while let Some(message) = socket.next().await {
        let message = message.context("event stream failed")?;

        let Message::Text(text) = message else {
            continue;
        };

        let frame: Value = serde_json::from_str(&text).context("malformed frame")?;

        match frame["type"].as_str() {
            Some("connection_ack") => {
                send(
                    &mut socket,
                    json!({
                        "id": "events",
                        "type": "subscribe",
                        "payload": { "query": EVENTS_SUBSCRIPTION, "variables": { "filter": filter } },
                    }),
                )
                .await?;

                eprintln!("subscribed to {filter}");
            }
            Some("next") => {
                if let Some(errors) = frame["payload"]["errors"].as_array() {
                    let messages: Vec<&str> = errors
                        .iter()
                        .filter_map(|error| error["message"].as_str())
                        .collect();

                    anyhow::bail!("subscription failed: {}", messages.join("; "));
                }

                print_event(&frame["payload"]["data"]["events"], as_json)?
            }
            Some("ping") => send(&mut socket, json!({ "type": "pong" })).await?,
            Some("error") => anyhow::bail!("subscription rejected: {}", frame["payload"]),
            Some("complete") => break,
            Some(other) => eprintln!("ignoring {other} frame"),
            None => eprintln!("ignoring untyped frame"),
        }
    }

    Ok(())
}

async fn send<S>(socket: &mut S, frame: Value) -> Result<()>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .context("failed to write to the event stream")
}

fn print_event(event: &Value, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", serde_json::to_string(event)?);
        return Ok(());
    }

    let kind = event["__typename"].as_str().unwrap_or("event");

    let mut fields = event.clone();
    if let Some(object) = fields.as_object_mut() {
        object.remove("__typename");
    }

    println!(
        "{} {kind} {fields}",
        chrono::Local::now().format("%H:%M:%S")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_schemes_become_websocket_schemes() {
        assert_eq!(
            websocket_url("https://home.anurag.sh", "/v1/graphql/ws"),
            "wss://home.anurag.sh/v1/graphql/ws"
        );
        assert_eq!(
            websocket_url("http://localhost:8080/", "/v1/lua/repl"),
            "ws://localhost:8080/v1/lua/repl"
        );
    }
}
