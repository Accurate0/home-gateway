pub mod subscriber;

use std::time::Duration;

use ractor::Actor;
use rand::RngExt;

use crate::adhoc::AnyAdhocCronTask;
use crate::adhoc::runner::{run_cron, run_pending};
use crate::adhoc::{cron_registry, registry};
use crate::event_bus::{Recipient, Subscription};
use crate::settings::AdhocSettings;
use crate::settings::enabled_state::EnabledState;
use crate::state::AppState;

use subscriber::AdhocSubscriber;

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
        task: &'static dyn AnyAdhocCronTask,
        settings: &AdhocSettings,
    ) {
        let config = task.config(&settings.tasks);

        if config.state() == EnabledState::Disabled {
            tracing::info!(
                "adhoc cron task {} is disabled, not scheduling",
                task.name()
            );
            return;
        }

        let schedule = config.schedule();

        let max_jitter = settings.cron_jitter();

        match schedule.time_until_next() {
            Ok(delay) => {
                let jitter =
                    Duration::from_secs(rand::rng().random_range(0..=max_jitter.as_secs()));
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
            tracing::info!("scheduling adhoc cron task {}", task.name());
            Self::schedule_next(&myself, task, &self.shared_actor_state.settings.adhoc);
        }

        let _ = myself.cast(AdhocTaskActorMessage::Recheck);

        Ok(AdhocTaskActorState {
            _subscription: subscription,
        })
    }

    #[tracing::instrument(
        parent = None,
        name = "adhoc-task-actor",
        skip(self, myself, message, _state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
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
                        Self::schedule_next(&myself, task, &self.shared_actor_state.settings.adhoc);
                    }
                    None => {
                        tracing::error!("adhoc cron task {name} fired but is no longer registered");
                        crate::tracing_context::record_current_error(&format!(
                            "adhoc cron task {name} is no longer registered"
                        ));
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
                        crate::tracing_context::record_current_error(&format!(
                            "adhoc cron task {name} is no longer registered"
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
