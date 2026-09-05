use crate::actors::health::ActorHealthRegistry;
use std::collections::HashMap;
use std::time::Duration;

use ractor::Actor;

use crate::actors::health::Lifecycle;
use crate::actors::manifest::{ACTORS, Spawned, find};
use crate::state::AppState;

const RESTART_BACKOFF_BASE: Duration = Duration::from_secs(1);
const RESTART_BACKOFF_MAX: Duration = Duration::from_secs(60);
const RESTART_BACKOFF_MAX_SHIFT: u32 = 16;
const HEALTHY_AFTER: Duration = Duration::from_secs(300);

pub enum RootMessage {
    Restart(String),
}

#[derive(Default)]
pub struct RootState {
    attempts: HashMap<String, u32>,
    started_at: HashMap<String, std::time::Instant>,
}

pub struct RootSupervisor {
    pub shared_actor_state: AppState,
}

impl RootSupervisor {
    async fn spawn(
        &self,
        myself: &ractor::ActorRef<RootMessage>,
        name: &str,
    ) -> Result<Spawned, ractor::ActorProcessingErr> {
        let Some(spec) = find(name) else {
            return Err(format!("no actor named `{name}` in the manifest").into());
        };

        if let Some(label) = spec.unmet_requirement(&self.shared_actor_state) {
            tracing::info!(actor = spec.name, "skipping {name}: {label} not configured");

            return Ok(Spawned::Skipped);
        }

        (spec.spawn)(myself.clone(), self.shared_actor_state.clone()).await
    }

    fn record_health(&self, name: &str, lifecycle: Lifecycle) {
        let Some(spec) = find(name) else {
            return;
        };

        self.shared_actor_state
            .handles
            .expect::<ActorHealthRegistry>()
            .record(spec.name, lifecycle);
    }

    fn attempts_for(&self, state: &mut RootState, name: &str) -> u32 {
        let healthy = state
            .started_at
            .get(name)
            .is_some_and(|started| started.elapsed() >= HEALTHY_AFTER);

        if healthy {
            state.attempts.remove(name);
        }

        let attempts = state.attempts.entry(name.to_owned()).or_insert(0);
        *attempts += 1;

        *attempts
    }
}

impl Actor for RootSupervisor {
    type Msg = RootMessage;
    type State = RootState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let mut state = RootState::default();

        for spec in ACTORS.iter().filter(|spec| spec.autostart) {
            match self.spawn(&myself, spec.name).await {
                Ok(Spawned::Started) => {
                    state
                        .started_at
                        .insert(spec.name.to_owned(), std::time::Instant::now());
                    self.record_health(spec.name, Lifecycle::Running);

                    tracing::debug!(actor = spec.name, "actor started");
                }
                Ok(Spawned::Skipped) => {
                    self.record_health(spec.name, Lifecycle::Skipped);

                    tracing::info!(actor = spec.name, "actor not configured, skipping")
                }
                Err(e) if spec.optional => {
                    self.record_health(spec.name, Lifecycle::Unavailable);

                    tracing::error!(
                        actor = spec.name,
                        "failed to start, continuing without it: {e}"
                    )
                }
                Err(e) => return Err(e),
            }
        }

        Ok(state)
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match &message {
            ractor::SupervisionEvent::ActorTerminated(who, _, _)
            | ractor::SupervisionEvent::ActorFailed(who, _) => {
                tracing::error!("actor {who:?} failed");
                if let ractor::SupervisionEvent::ActorFailed(_, panic) = &message {
                    tracing::error!("panic: {panic}");
                }

                let Some(name) = who.get_name() else {
                    tracing::error!("actor without name failed, cannot restart");
                    return Ok(());
                };

                let attempts = self.attempts_for(state, &name);
                let backoff = restart_backoff(attempts);

                self.record_health(&name, Lifecycle::Unavailable);
                crate::metrics::record_actor_restart(&name, "failed");
                tracing::warn!(actor = %name, attempts, ?backoff, "scheduling restart");

                myself.send_after(backoff, move || RootMessage::Restart(name));
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            RootMessage::Restart(name) => {
                tracing::info!(actor = %name, "restarting actor");

                match self.spawn(&myself, &name).await {
                    Ok(Spawned::Started) => {
                        state
                            .started_at
                            .insert(name.clone(), std::time::Instant::now());
                        self.record_health(&name, Lifecycle::Running);
                    }
                    Ok(Spawned::Skipped) => {
                        self.record_health(&name, Lifecycle::Skipped);

                        tracing::info!(actor = %name, "actor not configured, not restarting")
                    }
                    Err(e) => {
                        let attempts = self.attempts_for(state, &name);
                        let backoff = restart_backoff(attempts);

                        crate::metrics::record_actor_restart(&name, "respawn-failed");
                        tracing::error!(
                            actor = %name,
                            attempts,
                            ?backoff,
                            "failed to restart actor, retrying: {e}"
                        );

                        myself.send_after(backoff, move || RootMessage::Restart(name));
                    }
                }
            }
        }

        Ok(())
    }
}

fn restart_backoff(attempts: u32) -> Duration {
    RESTART_BACKOFF_BASE
        .saturating_mul(
            2u32.saturating_pow(attempts.saturating_sub(1).min(RESTART_BACKOFF_MAX_SHIFT)),
        )
        .min(RESTART_BACKOFF_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_backoff_grows_and_saturates() {
        assert_eq!(restart_backoff(1), RESTART_BACKOFF_BASE);
        assert_eq!(restart_backoff(2), RESTART_BACKOFF_BASE * 2);
        assert_eq!(restart_backoff(3), RESTART_BACKOFF_BASE * 4);
        assert_eq!(restart_backoff(u32::MAX), RESTART_BACKOFF_MAX);
    }

    #[test]
    fn restart_backoff_never_exceeds_the_cap() {
        for attempts in 1..64 {
            assert!(restart_backoff(attempts) <= RESTART_BACKOFF_MAX);
        }
    }
}
