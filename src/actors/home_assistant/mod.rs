use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use futures_util::{SinkExt, StreamExt};
use ractor::Actor;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

use crate::{
    event_bus::EventBusMessage, home_assistant::HomeAssistant, settings::EntitySettings,
    types::SharedActorState,
};

pub struct HomeAssistantActor {
    pub shared_actor_state: SharedActorState,
}

impl HomeAssistantActor {
    pub const NAME: &str = "home-assistant";

    const SUBSCRIBE_ID: u64 = 1;

    async fn run(&self, home_assistant: &HomeAssistant) -> Result<(), anyhow::Error> {
        let url = home_assistant.ws_url();
        tracing::info!("connecting to home assistant websocket at {url}");
        let (mut socket, _) = tokio_tungstenite::connect_async(&url).await?;
        let mut last_latest_state_write: HashMap<String, Instant> = HashMap::new();

        while let Some(message) = socket.next().await {
            let message = message?;
            let text = match message {
                WsMessage::Text(text) => text,
                WsMessage::Ping(payload) => {
                    socket.send(WsMessage::Pong(payload)).await?;
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
                    socket
                        .send(WsMessage::text(
                            json!({ "type": "auth", "access_token": home_assistant.token() })
                                .to_string(),
                        ))
                        .await?;
                }
                Some("auth_ok") => {
                    tracing::info!("home assistant authenticated, subscribing to state changes");
                    socket
                        .send(WsMessage::text(
                            json!({
                                "id": Self::SUBSCRIBE_ID,
                                "type": "subscribe_events",
                                "event_type": "state_changed",
                            })
                            .to_string(),
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

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::HomeAssistant {
                event_id,
                entity_id: entity_id.to_owned(),
                state,
            });
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
            sqlx::query!(
                "INSERT INTO home_assistant_events (event_id, entity_id, state) VALUES ($1, $2, $3)",
                event_id,
                entity_id,
                state,
            )
            .execute(&self.shared_actor_state.db)
            .await?;
        }

        if !write_latest_state {
            return Ok(());
        }

        sqlx::query!(
            r#"INSERT INTO latest_home_assistant_state (entity_id, state, event_id, updated_at) VALUES ($1, $2, $3, now())
            ON CONFLICT (entity_id)
            DO UPDATE SET
                state = EXCLUDED.state,
                event_id = EXCLUDED.event_id,
                updated_at = EXCLUDED.updated_at
            "#,
            entity_id,
            state,
            event_id,
        )
        .execute(&self.shared_actor_state.db)
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
        let Some(home_assistant) = self.shared_actor_state.home_assistant.clone() else {
            return Err(anyhow::anyhow!("home assistant is not configured").into());
        };

        let shared_actor_state = self.shared_actor_state.clone();
        tokio::spawn(async move {
            let actor = HomeAssistantActor { shared_actor_state };
            if let Err(e) = actor.run(&home_assistant).await {
                tracing::error!("home assistant websocket error: {e}");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
            myself.stop(Some("home assistant websocket disconnected".to_owned()));
        });

        Ok(())
    }
}
