use std::time::Duration;

use ractor::Actor;
use ractor::factory::{FactoryMessage, Job, JobOptions};
use tracing::Level;

use super::{ReconcilerMessage, ReconcilerWorker};
use crate::actors::system::rpc;
use crate::state::AppState;

pub enum ReconcilerSweeperMessage {
    Sweep,
}

pub struct ReconcilerSweeper {
    pub shared_actor_state: AppState,
}

impl ReconcilerSweeper {
    pub const NAME: &str = "reconciler-sweeper";

    async fn sweep(&self) -> Result<(), anyhow::Error> {
        let reconciler = &self.shared_actor_state.settings.reconciler;

        let claimed = self
            .shared_actor_state
            .repos
            .intent()
            .claim_due(reconciler.grace, reconciler.backoff, reconciler.batch_size)
            .await?;

        if claimed.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "claimed {} unconfirmed intent(s) to re-drive",
            claimed.len()
        );

        for intent in claimed {
            let job = FactoryMessage::Dispatch(Job {
                key: intent.address.clone(),
                msg: ReconcilerMessage::Retry(Box::new(intent)),
                options: JobOptions::default(),
                accepted: None,
            });

            if let Err(e) = rpc::cast(ReconcilerWorker::NAME, job) {
                tracing::warn!("failed to dispatch a reconciler job: {e}");
            }
        }

        Ok(())
    }
}

impl Actor for ReconcilerSweeper {
    type Msg = ReconcilerSweeperMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let reconciler = &self.shared_actor_state.settings.reconciler;

        let interval = reconciler
            .interval
            .to_std()
            .unwrap_or(Duration::from_secs(5));
        let _join_handle = myself.send_interval(interval, || ReconcilerSweeperMessage::Sweep);

        Ok(())
    }

    #[tracing::instrument(name = "reconciler-sweeper", skip(self, _myself, message, _state), level = Level::TRACE)]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ReconcilerSweeperMessage::Sweep => {
                if let Err(e) = self.sweep().await {
                    tracing::error!("reconciler sweep failed: {e}");
                }
            }
        }

        Ok(())
    }
}
