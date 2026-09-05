use std::time::Duration;

use ractor::Actor;
use rand::RngExt;
use tokio::sync::broadcast::error::RecvError;

use crate::adhoc::cron_task::AdhocCronTask;
use crate::adhoc::runner::{run_cron, run_pending};
use crate::adhoc::{cron_registry, registry};
use crate::integrations::feature_flag::ProviderEvent;
use crate::state::AppState;

const RESUBSCRIBE_BACKOFF: Duration = Duration::from_secs(30);
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

    fn watch_flags(myself: ractor::ActorRef<AdhocTaskActorMessage>, state: AppState) {
        tokio::spawn(async move {
            let mut events = match state.feature_flag_client.subscribe() {
                Some(events) => events,
                None => {
                    tracing::debug!("no live flag provider, adhoc watcher relies on the interval");

                    return;
                }
            };

            loop {
                match events.recv().await {
                    Ok(ProviderEvent::Ready) => {
                        tracing::info!("flag provider ready, rechecking adhoc queue");
                        let _ = myself.cast(AdhocTaskActorMessage::Recheck);
                    }
                    Ok(ProviderEvent::ConfigurationChanged { version }) => {
                        tracing::info!(
                            "flag configuration changed to version {version}, rechecking adhoc queue"
                        );
                        let _ = myself.cast(AdhocTaskActorMessage::Recheck);
                    }
                    Ok(ProviderEvent::Stale) => {
                        tracing::warn!("flag provider is stale, holding adhoc queue where it is");
                    }
                    Ok(ProviderEvent::Error) => {
                        tracing::warn!("flag provider errored, holding adhoc queue where it is");
                    }
                    Err(RecvError::Lagged(n)) => {
                        tracing::warn!("missed {n} flag events, rechecking adhoc queue");
                        let _ = myself.cast(AdhocTaskActorMessage::Recheck);
                    }
                    Err(RecvError::Closed) => {
                        tracing::warn!("flag event stream closed, resubscribing after backoff");
                        tokio::time::sleep(RESUBSCRIBE_BACKOFF).await;

                        match state.feature_flag_client.subscribe() {
                            Some(next) => {
                                events = next;
                                let _ = myself.cast(AdhocTaskActorMessage::Recheck);
                            }
                            None => {
                                tracing::error!(
                                    "flag provider is gone, adhoc watcher falling back to the interval"
                                );

                                return;
                            }
                        }
                    }
                }
            }
        });
    }
}

impl Actor for AdhocTaskActor {
    type Msg = AdhocTaskActorMessage;
    type State = ();
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
        Self::watch_flags(myself.clone(), self.shared_actor_state.clone());

        for task in cron_registry() {
            tracing::info!(
                "scheduling adhoc cron task {} on '{}'",
                task.name(),
                task.schedule().expression()
            );
            Self::schedule_next(&myself, task);
        }

        let _ = myself.cast(AdhocTaskActorMessage::Recheck);

        Ok(())
    }

    #[tracing::instrument(name = "adhoc-task-actor", skip(self, myself, message, _state))]
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
