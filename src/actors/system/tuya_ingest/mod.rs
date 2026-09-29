use std::collections::HashMap;

use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde_json::json;
use tracing::Instrument;
use uuid::Uuid;

use crate::device_registry::Transport;
use crate::integrations::tuya::DpsUpdate;
use crate::state::AppState;

use worker_state::WorkerState;

pub mod spawn;
mod worker_state;

pub enum Message {
    Dps(DpsUpdate),
    Connection { address: String, connected: bool },
}

pub struct TuyaIngest {
    pub shared_actor_state: AppState,
}

fn ingest_error(address: &str, kind: &'static str, error: &str) {
    let _errored = tracing::error_span!(
        parent: None,
        "tuya.ingest.error",
        force_sample = "true",
        address = %address,
        kind,
        otel.status_code = "ERROR",
        otel.status_message = %error,
    )
    .entered();
}

impl TuyaIngest {
    pub const NAME: &str = "tuya-ingest";

    async fn handle(&self, state: &mut WorkerState, message: Message) {
        match message {
            Message::Dps(update) => self.handle_dps(state, update).await,
            Message::Connection { address, connected } => {
                crate::device_registry::connection::record(
                    &self.shared_actor_state.devices,
                    self.shared_actor_state.repos.device(),
                    &self.shared_actor_state.event_bus,
                    Transport::Tuya,
                    &address,
                    connected,
                )
                .await
            }
        }
    }

    async fn handle_dps(&self, state: &mut WorkerState, update: DpsUpdate) {
        let DpsUpdate { address, dps } = update;

        let devices = &self.shared_actor_state.devices;

        let Some(device) = devices.decoded(&address) else {
            tracing::warn!("data points from unregistered tuya device {address}");
            return;
        };

        crate::device_registry::last_seen::record(
            devices,
            self.shared_actor_state.repos.device(),
            &address,
        )
        .await;

        if dps.is_empty() {
            tracing::trace!("tuya device {address} sent no data points");
            return;
        }

        let merged = state.dps.entry(address.clone()).or_default();
        merged.extend(dps.clone());

        let input = json!({
            "dps": merged,
            "changed": dps,
        });

        let reading = match device.profile.decode(&state.decoder, &input) {
            Ok(reading) => reading,
            Err(e) => {
                tracing::error!(
                    "failed to decode tuya data points on {address} with model {}: {e}",
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

impl Worker for TuyaIngest {
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

        let decoder = settings
            .model_sources
            .decoder(Transport::Tuya, &settings.lua)?;

        Ok(WorkerState {
            decoder,
            dps: HashMap::new(),
        })
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<String, Message>>,
        Job { key, msg, .. }: Job<String, Message>,
        state: &mut Self::State,
    ) -> Result<String, ActorProcessingErr> {
        let span = tracing::info_span!(parent: None, "tuya.ingest", address = %key);

        Self::handle(self, state, msg).instrument(span).await;

        Ok(key)
    }
}

pub struct TuyaIngestBuilder {
    pub shared_actor_state: AppState,
}

impl WorkerBuilder<TuyaIngest, ()> for TuyaIngestBuilder {
    fn build(&mut self, _wid: usize) -> (TuyaIngest, ()) {
        (
            TuyaIngest {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
