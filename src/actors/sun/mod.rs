//! Sun producer: turns sunrise/sunset into bus events.
//!
//! Scans the configured workflows for `Sun` triggers, and for each distinct
//! (transition, offset) pair schedules a one-shot timer that fires at the next
//! occurrence (the sun time shifted by the offset). On firing it publishes an
//! [`EventBusMessage::Sun`](crate::event_bus::EventBusMessage::Sun) and re-arms,
//! so dusk/dawn is just another event source with no special-casing in the
//! workflow machinery. Mirrors the [`crate::actors::system::cron::CronActor`] model.
//!
//! Every firing is recorded in `sun_event_fired`, so on startup a transition that
//! came due while the process was down is replayed once — provided it passed less
//! than `sun.catch_up_within` ago.

use chrono::{DateTime, TimeDelta, Utc};
use ractor::Actor;
use uuid::Uuid;

use crate::{event_bus::EventBusMessage, settings::TriggerMatcher, state::SharedActorState};

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

    async fn publish(
        &self,
        transition: SunTransition,
        offset: TimeDelta,
        at: DateTime<Utc>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Sun {
                event_id: Uuid::new_v4(),
                transition,
                offset,
            });

        sqlx::query!(
            "INSERT INTO sun_event_fired (transition, offset_seconds, fired_at) VALUES ($1, $2, $3) \
             ON CONFLICT (transition, offset_seconds) DO UPDATE SET fired_at = EXCLUDED.fired_at",
            transition.as_str(),
            offset.num_seconds(),
            at
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn last_fired(
        &self,
        transition: SunTransition,
        offset: TimeDelta,
    ) -> Result<Option<DateTime<Utc>>, ractor::ActorProcessingErr> {
        let row = sqlx::query!(
            "SELECT fired_at FROM sun_event_fired WHERE transition = $1 AND offset_seconds = $2",
            transition.as_str(),
            offset.num_seconds()
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(row.map(|row| row.fired_at))
    }

    async fn catch_up(
        &self,
        transition: SunTransition,
        offset: TimeDelta,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let now = Utc::now();
        let previous = calc::previous_transition_at(
            self.shared_actor_state.settings.location,
            now,
            transition,
            offset,
        );
        let last_fired = self.last_fired(transition, offset).await?;
        let grace = self.shared_actor_state.settings.sun.catch_up_within;

        if calc::should_catch_up(previous, last_fired, now, grace) {
            tracing::info!(
                "catching up missed sun {transition:?} (offset {}) from {previous}",
                crate::timedelta_format::humanize(offset)
            );
            self.publish(transition, offset, now).await?;
        } else {
            tracing::info!(
                "no catch-up for sun {transition:?} (offset {}): previous {previous}, last fired {last_fired:?}",
                crate::timedelta_format::humanize(offset)
            );
        }

        Ok(())
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
                    self.catch_up(*transition, *offset).await?;
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
                self.publish(transition, offset, Utc::now()).await?;
                self.schedule(&myself, transition, offset);
            }
        }
        Ok(())
    }
}
