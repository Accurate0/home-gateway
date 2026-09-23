use std::collections::HashMap;

use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde_json::json;
use tracing::Instrument;
use uuid::Uuid;

use crate::device_registry::Transport;
use crate::event_bus::EventBusMessage;
use crate::integrations::esphome_native_api::StateUpdate;
use crate::state::AppState;

use worker_state::WorkerState;

pub mod spawn;
mod worker_state;

pub enum Message {
    State(StateUpdate),
    Connection { address: String, connected: bool },
}

pub struct EsphomeNativeApiIngest {
    pub shared_actor_state: AppState,
}

fn ingest_error(address: &str, kind: &'static str, error: &str) {
    let _errored = tracing::error_span!(
        parent: None,
        "esphome_native_api.ingest.error",
        force_sample = "true",
        address = %address,
        kind,
        otel.status_code = "ERROR",
        otel.status_message = %error,
    )
    .entered();
}

impl EsphomeNativeApiIngest {
    pub const NAME: &str = "esphome-native-api-ingest";

    async fn handle(&self, state: &mut WorkerState, message: Message) {
        match message {
            Message::State(update) => self.handle_state(state, update).await,
            Message::Connection { address, connected } => {
                self.handle_connection(&address, connected)
            }
        }
    }

    fn handle_connection(&self, address: &str, connected: bool) {
        let devices = &self.shared_actor_state.devices;

        let Some(device_id) = devices.id_for_address(address) else {
            tracing::warn!("connection change for unregistered esphome node {address}");
            return;
        };

        match connected {
            true => tracing::info!("esphome node {device_id} ({address}) is connected"),
            false => tracing::warn!("esphome node {device_id} ({address}) is disconnected"),
        }

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::DeviceConnection {
                event_id: Uuid::new_v4(),
                device_id: device_id.to_owned(),
                transport: Transport::EsphomeNativeApi.to_string(),
                room: devices.room(address).map(str::to_owned),
                connected,
            });
    }

    async fn handle_state(&self, state: &mut WorkerState, update: StateUpdate) {
        let StateUpdate {
            address,
            domain,
            object_id,
            payload,
        } = update;

        let devices = &self.shared_actor_state.devices;

        let Some(device) = devices.decoded(&address) else {
            tracing::warn!("state from unregistered esphome node {address}");
            return;
        };

        crate::device_registry::last_seen::record(
            devices,
            self.shared_actor_state.repos.device(),
            &address,
        )
        .await;

        let entities = state.entities.entry(address.clone()).or_default();
        entities.insert(object_id.clone(), payload.clone());

        let input = json!({
            "domain": domain.to_string(),
            "object_id": object_id,
            "payload": payload,
            "entities": entities,
        });

        let reading = match device.profile.decode(&state.decoder, &input) {
            Ok(reading) => reading,
            Err(e) => {
                tracing::error!(
                    "failed to decode esphome {domain} entity {object_id} on {address} with model {}: {e}",
                    device.profile.slug
                );
                crate::tracing_context::record_current_error(&e.to_string());
                ingest_error(&address, "decode", &e.to_string());

                return;
            }
        };

        crate::decoding::dispatch(
            &self.shared_actor_state,
            Uuid::new_v4(),
            &device,
            &device.id,
            reading,
        )
        .await;
    }
}

impl Worker for EsphomeNativeApiIngest {
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

        let decoder = settings.model_sources.decoder(
            crate::device_registry::Transport::EsphomeNativeApi,
            &settings.lua,
        )?;

        Ok(WorkerState {
            decoder,
            entities: HashMap::new(),
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
            "esphome_native_api.ingest",
            address = %key,
        );

        Self::handle(self, state, msg).instrument(span).await;

        Ok(key)
    }
}

pub struct EsphomeNativeApiIngestBuilder {
    pub shared_actor_state: AppState,
}

impl WorkerBuilder<EsphomeNativeApiIngest, ()> for EsphomeNativeApiIngestBuilder {
    fn build(&mut self, _wid: usize) -> (EsphomeNativeApiIngest, ()) {
        (
            EsphomeNativeApiIngest {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
