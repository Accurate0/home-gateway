use std::{collections::HashMap, time::Instant};

use futures_util::{SinkExt, StreamExt};
use ractor::Actor;

use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

use crate::{
    event_bus::EventBusMessage, integrations::home_assistant::HomeAssistant,
    settings::EntitySettings, state::AppState,
};

pub struct HomeAssistantActor {
    pub shared_actor_state: AppState,
}

impl HomeAssistantActor {
    pub const NAME: &str = "home-assistant";

    const SUBSCRIBE_ID: u64 = 1;
    const GET_STATES_ID: u64 = 2;

    async fn run(&self, home_assistant: &HomeAssistant) -> Result<(), anyhow::Error> {
        let websocket = self.shared_actor_state.settings.home_assistant.websocket;
        let silence_timeout = websocket.silence_timeout();

        let url = home_assistant.ws_url();
        tracing::info!("connecting to home assistant websocket at {url}");
        let (socket, _) = tokio_tungstenite::connect_async(&url).await?;
        let (mut write, mut read) = socket.split();
        let mut last_latest_state_write: HashMap<String, Instant> = HashMap::new();

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
                                "id": Self::SUBSCRIBE_ID,
                                "type": "subscribe_events",
                                "event_type": "state_changed",
                            })
                            .to_string(),
                        ))
                        .await?;

                    write
                        .send(WsMessage::text(
                            json!({ "id": Self::GET_STATES_ID, "type": "get_states" }).to_string(),
                        ))
                        .await?;
                }
                Some("auth_invalid") => {
                    return Err(anyhow::anyhow!("home assistant rejected the access token"));
                }
                Some("event") => {
                    self.handle_event(&payload, &mut last_latest_state_write)
                        .await;
                }
                Some("result")
                    if payload.get("id").and_then(Value::as_u64) == Some(Self::GET_STATES_ID) =>
                {
                    self.seed_from_states(&payload).await;
                }
                _ => {}
            }
        }

        Err(anyhow::anyhow!("home assistant websocket stream ended"))
    }

    async fn handle_event(
        &self,
        payload: &Value,
        last_latest_state_write: &mut HashMap<String, Instant>,
    ) {
        let data = &payload["event"]["data"];
        let Some(entity_id) = data.get("entity_id").and_then(Value::as_str) else {
            return;
        };
        let state = data["new_state"]
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        let event_id = Uuid::new_v4();
        let entity = self
            .shared_actor_state
            .settings
            .home_assistant
            .for_entity(entity_id);

        let write_latest_state = entity.latest_state
            && match (
                entity.throttle.to_std(),
                last_latest_state_write.get(entity_id),
            ) {
                (Ok(throttle), Some(last)) => last.elapsed() >= throttle,
                _ => true,
            };

        if let Err(e) = self
            .save_to_db(event_id, entity_id, &state, &entity, write_latest_state)
            .await
        {
            tracing::error!("failed to persist home assistant state update: {e}");
        } else if write_latest_state {
            last_latest_state_write.insert(entity_id.to_owned(), Instant::now());
        }

        self.forward_decoded(
            event_id,
            entity_id,
            &state,
            &data["new_state"]["attributes"],
        )
        .await;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::HomeAssistant {
                event_id,
                entity_id: entity_id.to_owned(),
                state,
            });
    }

    async fn seed_from_states(&self, payload: &Value) {
        let Some(states) = payload.get("result").and_then(Value::as_array) else {
            tracing::warn!("home assistant get_states reply carried no result array");
            return;
        };

        let devices = &self.shared_actor_state.devices;

        for entity in states {
            let Some(entity_id) = entity.get("entity_id").and_then(Value::as_str) else {
                continue;
            };

            if devices.home_assistant_device(entity_id).is_none() {
                continue;
            }

            let state = entity
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or_default();

            tracing::info!("seeding {entity_id} from get_states ({state})");

            self.forward_decoded(Uuid::new_v4(), entity_id, state, &entity["attributes"])
                .await;
        }
    }

    async fn forward_decoded(
        &self,
        event_id: Uuid,
        entity_id: &str,
        state: &str,
        attributes: &Value,
    ) {
        let devices = &self.shared_actor_state.devices;

        let Some(device) = devices.home_assistant_device(entity_id) else {
            return;
        };

        crate::device_registry::last_seen::record(
            devices,
            self.shared_actor_state.repos.device(),
            &device.address,
        )
        .await;

        let entity = json!({
            "entity_id": entity_id,
            "state": state,
            "attributes": attributes,
        });

        let reading = match device.profile.decode(&entity) {
            Ok(reading) => reading,
            Err(e) => {
                tracing::error!(
                    "failed to decode home assistant entity {entity_id} with model {}: {e}",
                    device.profile.slug
                );
                crate::tracing_context::record_current_error(&e.to_string());

                return;
            }
        };

        let friendly_name = attributes
            .get("friendly_name")
            .and_then(Value::as_str)
            .unwrap_or(&device.id);

        crate::decoding::dispatch(
            &self.shared_actor_state,
            event_id,
            device,
            friendly_name,
            reading,
        )
        .await;
    }

    async fn save_to_db(
        &self,
        event_id: Uuid,
        entity_id: &str,
        state: &str,
        entity: &EntitySettings,
        write_latest_state: bool,
    ) -> Result<(), anyhow::Error> {
        if entity.log {
            self.shared_actor_state
                .repos
                .home_assistant()
                .append_event(event_id, entity_id, state)
                .await?;
        }

        if !write_latest_state {
            return Ok(());
        }

        self.shared_actor_state
            .repos
            .home_assistant()
            .upsert_latest(event_id, entity_id, state)
            .await?;

        Ok(())
    }
}

impl Actor for HomeAssistantActor {
    type Msg = ();
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let Some(home_assistant) = self
            .shared_actor_state
            .handles
            .get::<HomeAssistant>()
            .cloned()
        else {
            return Err(anyhow::anyhow!("home assistant is not configured").into());
        };

        let reconnect_delay = self
            .shared_actor_state
            .settings
            .home_assistant
            .websocket
            .reconnect_delay();

        let shared_actor_state = self.shared_actor_state.clone();
        tokio::spawn(async move {
            let actor = HomeAssistantActor { shared_actor_state };
            if let Err(e) = actor.run(&home_assistant).await {
                tracing::error!("home assistant websocket error: {e}");
            }
            tokio::time::sleep(reconnect_delay).await;
            myself.stop(Some("home assistant websocket disconnected".to_owned()));
        });

        Ok(())
    }
}
