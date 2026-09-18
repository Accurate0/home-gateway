use futures_util::{SinkExt, StreamExt};
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;

use super::HomeAssistant;
use crate::actors::system::home_assistant_ingest::{self, HomeAssistantIngest};
use crate::settings::HomeAssistantWebsocketSettings;

const SUBSCRIBE_ID: u64 = 1;
const GET_STATES_ID: u64 = 2;

fn ingest_actor() -> Option<ActorRef<FactoryMessage<String, home_assistant_ingest::Message>>> {
    ractor::registry::where_is(HomeAssistantIngest::NAME).map(ActorRef::from)
}

fn dispatch(entity_id: String, msg: home_assistant_ingest::Message) {
    let Some(actor) = ingest_actor() else {
        tracing::error!("home assistant ingest actor is not registered, dropping {entity_id}");
        return;
    };

    let response = actor.send_message(FactoryMessage::Dispatch(Job {
        key: entity_id,
        msg,
        options: JobOptions::default(),
        accepted: None,
    }));

    if let Err(e) = response {
        tracing::error!("error sending to home assistant ingest actor: {e}");
    }
}

pub async fn process_events(
    home_assistant: HomeAssistant,
    websocket: HomeAssistantWebsocketSettings,
    cancellation_token: CancellationToken,
) {
    loop {
        tokio::select! {
            result = read_events(&home_assistant, websocket) => {
                if let Err(e) = result {
                    tracing::error!(
                        "home assistant websocket error, reconnecting in {:?}: {e}",
                        websocket.reconnect_delay()
                    );
                }

                tokio::time::sleep(websocket.reconnect_delay()).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("home assistant websocket cancellation requested");
                return;
            }
        }
    }
}

async fn read_events(
    home_assistant: &HomeAssistant,
    websocket: HomeAssistantWebsocketSettings,
) -> Result<(), anyhow::Error> {
    let silence_timeout = websocket.silence_timeout();

    let url = home_assistant.ws_url();
    tracing::info!("connecting to home assistant websocket at {url}");
    let (socket, _) = tokio_tungstenite::connect_async(&url).await?;
    let (mut write, mut read) = socket.split();

    let mut keep_alive = tokio::time::interval(websocket.keep_alive());
    keep_alive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    keep_alive.tick().await;

    loop {
        let message = tokio::select! {
            _ = keep_alive.tick() => {
                write.send(WsMessage::Ping(Vec::new().into())).await?;
                continue;
            }
            message = tokio::time::timeout(silence_timeout, read.next()) => message,
        };

        let message = match message {
            Ok(Some(message)) => message?,
            Ok(None) => break,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "no home assistant message for {silence_timeout:?}, assuming the connection is dead"
                ));
            }
        };

        let text = match message {
            WsMessage::Text(text) => text,
            WsMessage::Ping(payload) => {
                write.send(WsMessage::Pong(payload)).await?;
                continue;
            }
            WsMessage::Close(_) => {
                return Err(anyhow::anyhow!("home assistant closed the connection"));
            }
            _ => continue,
        };

        let payload: Value = match serde_json::from_str(&text) {
            Ok(payload) => payload,
            Err(e) => {
                tracing::warn!("failed to parse home assistant message: {e}");
                continue;
            }
        };

        match payload.get("type").and_then(Value::as_str) {
            Some("auth_required") => {
                write
                    .send(WsMessage::text(
                        json!({ "type": "auth", "access_token": home_assistant.token() })
                            .to_string(),
                    ))
                    .await?;
            }
            Some("auth_ok") => {
                tracing::info!("home assistant authenticated, subscribing to state changes");
                write
                    .send(WsMessage::text(
                        json!({
                            "id": SUBSCRIBE_ID,
                            "type": "subscribe_events",
                            "event_type": "state_changed",
                        })
                        .to_string(),
                    ))
                    .await?;

                write
                    .send(WsMessage::text(
                        json!({ "id": GET_STATES_ID, "type": "get_states" }).to_string(),
                    ))
                    .await?;
            }
            Some("auth_invalid") => {
                return Err(anyhow::anyhow!("home assistant rejected the access token"));
            }
            Some("event") => dispatch_event(&payload),
            Some("result")
                if payload.get("id").and_then(Value::as_u64) == Some(GET_STATES_ID) =>
            {
                dispatch_states(&payload);
            }
            _ => {}
        }
    }

    Err(anyhow::anyhow!("home assistant websocket stream ended"))
}

fn dispatch_event(payload: &Value) {
    let data = &payload["event"]["data"];

    let Some(entity_id) = data.get("entity_id").and_then(Value::as_str) else {
        return;
    };

    let new_state = &data["new_state"];

    let state = new_state
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    dispatch(
        entity_id.to_owned(),
        home_assistant_ingest::Message::StateChanged {
            entity_id: entity_id.to_owned(),
            state,
            attributes: new_state["attributes"].clone(),
        },
    );
}

fn dispatch_states(payload: &Value) {
    let Some(states) = payload.get("result").and_then(Value::as_array) else {
        tracing::warn!("home assistant get_states reply carried no result array");
        return;
    };

    for entity in states {
        let Some(entity_id) = entity.get("entity_id").and_then(Value::as_str) else {
            continue;
        };

        let state = entity
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        dispatch(
            entity_id.to_owned(),
            home_assistant_ingest::Message::Seed {
                entity_id: entity_id.to_owned(),
                state,
                attributes: entity["attributes"].clone(),
            },
        );
    }
}
