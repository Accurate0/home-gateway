pub mod spawn;
pub mod sweeper;

use ractor::factory::{Job, WorkerBuilder, WorkerId};
use uuid::Uuid;

use crate::actors::devices::light::{LightHandler, LightHandlerMessage};
use crate::actors::system::rpc;
use crate::event_bus::EventBusMessage;
use crate::repo::intent::{DeviceIntent, IntentAttributes, IntentStatus};
use crate::state::AppState;

pub use spawn::spawn_reconciler;
pub use sweeper::{ReconcilerSweeper, ReconcilerSweeperMessage};

pub enum ReconcilerMessage {
    Retry(Box<DeviceIntent>),
}

pub struct ReconcilerWorker {
    shared_actor_state: AppState,
}

impl ReconcilerWorker {
    pub const NAME: &str = "reconciler";

    async fn retry(&self, intent: DeviceIntent) -> Result<(), anyhow::Error> {
        let max_attempts = self.shared_actor_state.settings.reconciler.max_attempts;

        if intent.attempts > max_attempts {
            self.give_up(intent).await;

            return Ok(());
        }

        tracing::info!(
            "re-driving intent {} for {} (attempt {}/{max_attempts})",
            intent.id,
            intent.address,
            intent.attempts,
        );

        if intent.attributes.is_empty() {
            tracing::warn!(
                "intent {} for {} has no re-drivable attributes, failing it",
                intent.id,
                intent.address,
            );
            self.give_up(intent).await;

            return Ok(());
        }

        match &intent.attributes {
            IntentAttributes::Light(attributes) => {
                crate::metrics::record_reconciler_retry(intent.kind().as_str());

                rpc::cast_factory(
                    LightHandler::NAME,
                    LightHandlerMessage::Reapply {
                        ieee_addr: intent.address.clone(),
                        attributes: Box::new(attributes.clone()),
                    },
                )?;

                Ok(())
            }
            IntentAttributes::SmartSwitch(_) => {
                tracing::warn!(
                    "intent {} for {} has no re-drive path for kind `{}`, failing it",
                    intent.id,
                    intent.address,
                    intent.kind().as_str(),
                );
                self.give_up(intent).await;

                Ok(())
            }
        }
    }

    async fn give_up(&self, intent: DeviceIntent) {
        tracing::error!(
            "intent {} for {} was never confirmed after {} attempts",
            intent.id,
            intent.address,
            intent.attempts,
        );
        crate::tracing_context::record_current_error("intent was never confirmed");
        crate::metrics::record_reconciler_give_up(intent.kind().as_str(), &intent.address);

        if let Err(e) = self
            .shared_actor_state
            .repos
            .intent()
            .settle(&[intent.id], IntentStatus::Failed)
            .await
        {
            tracing::warn!("failed to mark intent {} as failed: {e}", intent.id);
        }

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::CommandFailed {
                event_id: Uuid::new_v4(),
                kind: intent.kind(),
                device_id: self
                    .shared_actor_state
                    .devices
                    .id_for_address(&intent.address)
                    .map(str::to_owned),
                address: intent.address,
                attributes: intent.attributes,
                attempts: intent.attempts,
            });
    }
}

impl ractor::factory::Worker for ReconcilerWorker {
    type Key = String;
    type Message = ReconcilerMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ractor::ActorRef<ractor::factory::FactoryMessage<String, ReconcilerMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(
        parent = None,
        name = "reconciler-worker",
        skip(self, _wid, _factory, msg, _state),
        fields(otel.status_code = tracing::field::Empty, otel.status_message = tracing::field::Empty)
    )]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ractor::ActorRef<ractor::factory::FactoryMessage<String, ReconcilerMessage>>,
        Job { key, msg, .. }: Job<String, ReconcilerMessage>,
        _state: &mut Self::State,
    ) -> Result<String, ractor::ActorProcessingErr> {
        match msg {
            ReconcilerMessage::Retry(intent) => {
                let id = intent.id;

                if let Err(e) = self.retry(*intent).await {
                    tracing::error!("failed to re-drive intent {id}: {e}");
                }
            }
        }

        Ok(key)
    }
}

pub struct ReconcilerWorkerBuilder {
    pub shared_actor_state: AppState,
}

impl WorkerBuilder<ReconcilerWorker, ()> for ReconcilerWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (ReconcilerWorker, ()) {
        (
            ReconcilerWorker {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
