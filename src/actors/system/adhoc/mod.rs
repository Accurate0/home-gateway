pub mod subscriber;

use std::time::Duration;

use ractor::Actor;
use rand::RngExt;

use crate::adhoc::cron_task::AdhocCronTask;
use crate::adhoc::runner::{run_cron, run_pending};
use crate::adhoc::{cron_registry, registry};
use crate::event_bus::{Recipient, Subscription};
use crate::state::AppState;

use subscriber::AdhocSubscriber;

const CRON_JITTER_SECS: u64 = 60;

pub enum AdhocTaskActorMessage {
    Recheck,
    CronFire { name: &'static str },
    RunPending,
    RunCron { name: &'static str },
}

pub struct AdhocTaskActor {
    pub shared_actor_state: AppState,
}

impl AdhocTaskActor {
    pub const NAME: &str = "adhoc_task";

    fn schedule_next(
        myself: &ractor::ActorRef<AdhocTaskActorMessage>,
        task: &'static dyn AdhocCronTask,
    ) {
        match task.schedule().time_until_next() {
            Ok(delay) => {
                let jitter = Duration::from_secs(rand::rng().random_range(0..=CRON_JITTER_SECS));
                let name = task.name();

                tracing::debug!(
                    "adhoc cron task {name} firing in {}s (including {}s jitter)",
                    (delay + jitter).as_secs(),
                    jitter.as_secs()
                );
                myself.send_after(delay + jitter, move || AdhocTaskActorMessage::CronFire {
                    name,
                });
            }
            Err(e) => {
                tracing::error!(
                    "adhoc cron task {} has no next occurrence, not scheduling: {e}",
                    task.name()
                );
            }
        }
    }
}

#[derive(Default)]
pub struct AdhocTaskActorState {
    _subscription: Subscription,
}

impl Actor for AdhocTaskActor {
    type Msg = AdhocTaskActorMessage;
    type State = AdhocTaskActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let interval = self
            .shared_actor_state
            .settings
            .adhoc
            .recheck_interval
            .to_std()
            .unwrap_or(Duration::from_secs(900));

        myself.send_interval(interval, || AdhocTaskActorMessage::Recheck);

        let subscription = self.shared_actor_state.event_bus.register(
            Self::NAME,
            Recipient::Actor(myself.clone()),
            AdhocSubscriber,
        );

        for task in cron_registry() {
            tracing::info!(
                "scheduling adhoc cron task {} on '{}'",
                task.name(),
                task.schedule().expression()
            );
            Self::schedule_next(&myself, task);
        }

        let _ = myself.cast(AdhocTaskActorMessage::Recheck);

        Ok(AdhocTaskActorState {
            _subscription: subscription,
        })
    }

    #[tracing::instrument(parent = None, name = "adhoc-task-actor", skip(self, myself, message, _state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            AdhocTaskActorMessage::Recheck if registry().is_empty() => {
                tracing::trace!("no adhoc tasks registered");
            }
            AdhocTaskActorMessage::Recheck => {
                run_pending(&self.shared_actor_state, false).await;
            }
            AdhocTaskActorMessage::RunPending => {
                tracing::info!("draining the adhoc queue on demand");
                run_pending(&self.shared_actor_state, true).await;
            }
            AdhocTaskActorMessage::CronFire { name } => {
                match cron_registry().into_iter().find(|task| task.name() == name) {
                    Some(task) => {
                        run_cron(&self.shared_actor_state, task, false).await;
                        Self::schedule_next(&myself, task);
                    }
                    None => {
                        tracing::error!("adhoc cron task {name} fired but is no longer registered");
                    }
                }
            }
            AdhocTaskActorMessage::RunCron { name } => {
                match cron_registry().into_iter().find(|task| task.name() == name) {
                    Some(task) => {
                        tracing::info!("running adhoc cron task {name} on demand");
                        run_cron(&self.shared_actor_state, task, true).await;
                    }
                    None => {
                        tracing::error!("adhoc cron task {name} is no longer registered");
                    }
                }
            }
        }

        Ok(())
    }
}
