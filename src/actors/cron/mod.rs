//! Cron producer: turns time into bus events.
//!
//! Reads every `Cron` trigger from the config, schedules its next firing, and
//! publishes an [`EventBusMessage::Cron`](crate::event_bus::EventBusMessage::Cron)
//! when due. The dispatcher matches that event against the trigger by name and
//! runs its workflow — so a recurring schedule is just another event source,
//! with no special-casing in the workflow machinery.

use ractor::Actor;
use uuid::Uuid;

use crate::{event_bus::EventBusMessage, settings::TriggerMatcher, types::SharedActorState};

use schedule::CronSchedule;

pub mod schedule;

pub enum CronActorMessage {
    /// A scheduled trigger came due. Carries the schedule so the actor can
    /// re-arm itself for the next occurrence after publishing.
    Fire {
        name: String,
        schedule: CronSchedule,
    },
}

pub struct CronActor {
    pub shared_actor_state: SharedActorState,
}

impl CronActor {
    pub const NAME: &str = "cron";

    /// Arm a one-shot timer for the schedule's next occurrence. Re-called after
    /// each firing so the schedule repeats. Logs and stops re-arming if the
    /// schedule (already validated at load) somehow yields no next time.
    fn schedule_next(
        myself: &ractor::ActorRef<CronActorMessage>,
        name: String,
        schedule: CronSchedule,
    ) {
        match schedule.time_until_next() {
            Ok(delay) => {
                myself.send_after(delay, move || CronActorMessage::Fire { name, schedule });
            }
            Err(e) => {
                tracing::error!("cron trigger '{name}' has no next occurrence: {e}");
            }
        }
    }
}

impl Actor for CronActor {
    type Msg = CronActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let settings = &self.shared_actor_state.settings;
        for trigger in &settings.triggers {
            if !trigger.enabled {
                continue;
            }
            if let TriggerMatcher::Cron { schedule } = &trigger.on {
                Self::schedule_next(&myself, trigger.name.clone(), schedule.as_ref().clone());
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "cron-actor", skip(self, myself, message, _state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            CronActorMessage::Fire { name, schedule } => {
                self.shared_actor_state
                    .event_bus
                    .publish(EventBusMessage::Cron {
                        event_id: Uuid::new_v4(),
                        name: name.clone(),
                    });

                Self::schedule_next(&myself, name, schedule);
            }
        }

        Ok(())
    }
}
