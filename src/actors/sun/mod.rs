//! Sun producer: turns sunrise/sunset into bus events.
//!
//! Scans the configured workflows for `Sun` triggers, and for each distinct
//! (transition, offset) pair schedules a one-shot timer that fires at the next
//! occurrence (the sun time shifted by the offset). On firing it publishes an
//! [`EventBusMessage::Sun`](crate::event_bus::EventBusMessage::Sun) and re-arms,
//! so dusk/dawn is just another event source with no special-casing in the
//! workflow machinery. Mirrors the [`crate::actors::cron::CronActor`] model.

use chrono::{TimeDelta, Utc};
use ractor::Actor;
use uuid::Uuid;

use crate::{event_bus::EventBusMessage, settings::TriggerMatcher, types::SharedActorState};

use calc::SunTransition;

pub mod calc;

pub enum SunActorMessage {
    Fire {
        transition: SunTransition,
        offset: TimeDelta,
    },
}

pub struct SunActor {
    pub shared_actor_state: SharedActorState,
}

impl SunActor {
    pub const NAME: &str = "sun";

    fn schedule(
        &self,
        myself: &ractor::ActorRef<SunActorMessage>,
        transition: SunTransition,
        offset: TimeDelta,
    ) {
        let location = self.shared_actor_state.settings.location;
        let delay = calc::next_transition(location, Utc::now(), transition, offset);
        tracing::info!(
            "next sun {transition:?} (offset {}) in {delay:?}",
            crate::timedelta_format::humanize(offset)
        );
        myself.send_after(delay, move || SunActorMessage::Fire { transition, offset });
    }
}

impl Actor for SunActor {
    type Msg = SunActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let mut scheduled: Vec<(SunTransition, TimeDelta)> = Vec::new();
        for workflow in self.shared_actor_state.settings.workflows.values() {
            if !workflow.enabled {
                continue;
            }
            if let Some(TriggerMatcher::Sun { transition, offset }) = workflow.on() {
                let pair = (*transition, *offset);
                if !scheduled.contains(&pair) {
                    scheduled.push(pair);
                    self.schedule(&myself, *transition, *offset);
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(name = "sun-actor", skip(self, myself, message, _state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SunActorMessage::Fire { transition, offset } => {
                self.shared_actor_state
                    .event_bus
                    .publish(EventBusMessage::Sun {
                        event_id: Uuid::new_v4(),
                        transition,
                        offset,
                    });
                self.schedule(&myself, transition, offset);
            }
        }
        Ok(())
    }
}
