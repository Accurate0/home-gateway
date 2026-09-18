use std::collections::HashMap;
use std::time::Instant;

use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde_json::{Value, json};
use tracing::Instrument;
use uuid::Uuid;

use crate::{
    event_bus::EventBusMessage, lua::LuaDecoder, settings::EntitySettings, state::AppState,
};

use worker_state::WorkerState;

pub mod spawn;
mod worker_state;

pub enum Message {
    StateChanged {
        entity_id: String,
        state: String,
        attributes: Value,
    },
    Seed {
        entity_id: String,
        state: String,
        attributes: Value,
    },
}

impl Message {
    fn kind(&self) -> &'static str {
        match self {
            Message::StateChanged { .. } => "state_changed",
            Message::Seed { .. } => "seed",
        }
    }
}

pub struct HomeAssistantIngest {
    pub shared_actor_state: AppState,
}

impl HomeAssistantIngest {
    pub const NAME: &str = "home-assistant-ingest";

    async fn handle(&self, state: &mut WorkerState, message: Message) {
        match message {
            Message::StateChanged {
                entity_id,
                state: entity_state,
                attributes,
            } => {
                self.handle_state_changed(state, &entity_id, entity_state, &attributes)
                    .await;
            }
            Message::Seed {
                entity_id,
                state: entity_state,
                attributes,
            } => {
                if self
                    .shared_actor_state
                    .devices
                    .home_assistant_device(&entity_id)
                    .is_none()
                {
                    return;
                }

                tracing::info!("seeding {entity_id} from get_states ({entity_state})");

                self.forward_decoded(
                    &state.decoder,
                    Uuid::new_v4(),
                    &entity_id,
                    &entity_state,
                    &attributes,
                )
                .await;
            }
        }
    }

    async fn handle_state_changed(
        &self,
        state: &mut WorkerState,
        entity_id: &str,
        entity_state: String,
        attributes: &Value,
    ) {
        let event_id = Uuid::new_v4();
        let entity = self
            .shared_actor_state
            .settings
            .home_assistant
            .for_entity(entity_id);

        let write_latest_state = entity.latest_state
            && match (
                entity.throttle.to_std(),
                state.last_latest_state_write.get(entity_id),
            ) {
                (Ok(throttle), Some(last)) => last.elapsed() >= throttle,
                _ => true,
            };

        if let Err(e) = self
            .save_to_db(event_id, entity_id, &entity_state, &entity, write_latest_state)
            .await
        {
            tracing::error!("failed to persist home assistant state update: {e}");
        } else if write_latest_state {
            state
                .last_latest_state_write
                .insert(entity_id.to_owned(), Instant::now());
        }

        self.forward_decoded(&state.decoder, event_id, entity_id, &entity_state, attributes)
            .await;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::HomeAssistant {
                event_id,
                entity_id: entity_id.to_owned(),
                state: entity_state,
            });
    }

    async fn forward_decoded(
        &self,
        decoder: &LuaDecoder,
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

        let reading = match device.profile.decode(decoder, &entity) {
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

impl Worker for HomeAssistantIngest {
    type Key = String;
    type Message = Message;
    type State = WorkerState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<String, Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let settings = &self.shared_actor_state.settings;

        let decoder = LuaDecoder::load(
            "home_assistant",
            &settings.model_sources.home_assistant,
            &settings.lua,
        )?;

        Ok(WorkerState {
            decoder,
            last_latest_state_write: HashMap::new(),
        })
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<String, Message>>,
        Job { key, msg, .. }: Job<String, Message>,
        state: &mut Self::State,
    ) -> Result<String, ActorProcessingErr> {
        let span = tracing::info_span!(
            parent: None,
            "home_assistant.ingest",
            entity_id = %key,
            kind = msg.kind(),
        );

        Self::handle(self, state, msg).instrument(span).await;

        Ok(key)
    }
}

pub struct HomeAssistantIngestBuilder {
    pub shared_actor_state: AppState,
}

impl WorkerBuilder<HomeAssistantIngest, ()> for HomeAssistantIngestBuilder {
    fn build(&mut self, _wid: usize) -> (HomeAssistantIngest, ()) {
        (
            HomeAssistantIngest {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
