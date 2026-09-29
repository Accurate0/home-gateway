use std::time::Duration;

use chrono::{DateTime, Utc};
use chrono_tz::Australia::Perth;
use ractor::Actor;
use uuid::Uuid;

use crate::event_bus::EventBusMessage;
use crate::integrations::s3::S3;
use crate::integrations::transperth::{
    Departure, RouteDepartures, TIMETABLE_KEY, Transperth, gtfs::TimetableIndex,
};
use crate::state::AppState;

const FALLBACK_REFRESH: Duration = Duration::from_secs(900);

pub enum TransperthMessage {
    LoadTimetable,
    RefreshRoutes,
}

#[derive(Default)]
pub struct TransperthActorState {
    refresh_scheduled: bool,
}

pub struct TransperthActor {
    pub shared_actor_state: AppState,
    pub transperth: Transperth,
}

impl TransperthActor {
    pub const NAME: &str = "transperth";

    async fn load_timetable(&self) -> Result<(), ractor::ActorProcessingErr> {
        let payload = match self
            .shared_actor_state
            .handles
            .expect::<S3>()
            .get_object(TIMETABLE_KEY)
            .await
        {
            Ok(payload) => payload,
            Err(error) => {
                tracing::warn!(
                    "transperth timetable unavailable at {TIMETABLE_KEY}, run the refresh_transperth_timetable task: {error}"
                );
                return Ok(());
            }
        };

        let index: TimetableIndex = serde_json::from_slice(&payload)?;

        tracing::info!(
            "transperth timetable loaded ({} trips, {} stops)",
            index.trips.len(),
            index.stops.len()
        );

        self.transperth.store_index(index);

        Ok(())
    }

    async fn refresh(&self) {
        if !self.transperth.has_index() {
            tracing::debug!("transperth has no timetable index, skipping refresh");
            return;
        }

        let started = std::time::Instant::now();
        let now = Utc::now();
        match self.transperth.refresh_routes(now).await {
            Ok(routes) => {
                tracing::debug!("transperth routes refreshed");
                crate::metrics::record_integration_poll("transperth", "success", started.elapsed());

                for route in routes {
                    self.publish(&route, now);
                }
            }
            Err(error) => {
                tracing::warn!("transperth route refresh failed: {error}");
                crate::metrics::record_integration_poll("transperth", "error", started.elapsed());
            }
        }
    }

    fn publish(&self, route: &RouteDepartures, now: DateTime<Utc>) {
        let next = route.departures.first();

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Transperth {
                event_id: Uuid::new_v4(),
                route_id: route.id.clone(),
                origin: route.origin.clone(),
                destination: route.destination.clone(),
                departures: i32::try_from(route.departures.len()).unwrap_or(i32::MAX),
                next_line: next.map(|departure| departure.line.clone()),
                next_headsign: next.map(|departure| departure.headsign.clone()),
                next_minutes_away: next
                    .and_then(|departure| i32::try_from(departure.minutes_away(now)).ok()),
                next_delay_minutes: next
                    .and_then(Departure::delay_minutes)
                    .and_then(|minutes| i32::try_from(minutes).ok()),
            });
    }

    fn next_refresh(&self) -> Duration {
        let settings = &self.shared_actor_state.settings.integrations.transperth;
        let now = Utc::now().with_timezone(&Perth).time();

        settings
            .refresh_for(now)
            .to_std()
            .unwrap_or(FALLBACK_REFRESH)
    }
}

impl Actor for TransperthActor {
    type Msg = TransperthMessage;
    type State = TransperthActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        myself.send_message(TransperthMessage::LoadTimetable)?;

        Ok(TransperthActorState::default())
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            TransperthMessage::LoadTimetable => {
                if let Err(error) = self.load_timetable().await {
                    tracing::error!("transperth timetable load failed: {error}");
                }

                self.refresh().await;

                if !state.refresh_scheduled {
                    state.refresh_scheduled = true;
                    myself.send_after(self.next_refresh(), || TransperthMessage::RefreshRoutes);
                }
            }

            TransperthMessage::RefreshRoutes => {
                self.refresh().await;

                myself.send_after(self.next_refresh(), || TransperthMessage::RefreshRoutes);
            }
        }

        Ok(())
    }
}
