use chrono::TimeZone;
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use chrono_tz::Australia::Perth;

use crate::settings::TransperthRoute;

use super::gtfs::{TimetableIndex, service_datetime};
use super::{Departure, station_stop_ids};

pub fn build(
    index: &TimetableIndex,
    routes: &TransperthRoute,
    now: DateTime<Utc>,
    horizon: TimeDelta,
    service_dates: &[NaiveDate],
) -> Vec<Departure> {
    let origin = station_stop_ids(index, &routes.from);
    let destination = station_stop_ids(index, &routes.to);

    if origin.is_empty() || destination.is_empty() {
        tracing::warn!(
            "transperth routes {} matched no stops ({} -> {})",
            routes.id,
            routes.from,
            routes.to
        );
        return Vec::new();
    }

    let cutoff = now + horizon;
    let mut departures = Vec::new();

    for (trip_id, stop_times) in &index.stop_times {
        let Some(trip) = index.trips.get(trip_id) else {
            continue;
        };

        let Some(boarding) = stop_times
            .iter()
            .position(|stop_time| origin.contains(&stop_time.stop_id) && stop_time.pickup_allowed)
        else {
            continue;
        };

        let alights = stop_times
            .iter()
            .skip(boarding + 1)
            .any(|stop_time| destination.contains(&stop_time.stop_id) && stop_time.dropoff_allowed);

        if !alights {
            continue;
        }

        let stop_time = &stop_times[boarding];

        for date in service_dates {
            let runs = index
                .services
                .get(&trip.service_id)
                .is_some_and(|service| service.runs_on(*date));

            if !runs {
                continue;
            }

            let Some(naive) = service_datetime(*date, stop_time.departure_seconds) else {
                continue;
            };

            let Some(local) = Perth.from_local_datetime(&naive).earliest() else {
                continue;
            };

            let scheduled_departure = local.with_timezone(&Utc);

            if scheduled_departure < now || scheduled_departure > cutoff {
                continue;
            }

            departures.push(Departure {
                trip_id: trip.trip_id.clone(),
                line: trip.line.clone(),
                headsign: trip.headsign.clone(),
                platform: index
                    .stops
                    .get(&stop_time.stop_id)
                    .and_then(|stop| stop.platform_code.clone()),
                scheduled_departure,
                live_departure: None,
            });
        }
    }

    departures.sort_by_key(|departure| departure.scheduled_departure);
    departures.truncate(routes.limit);

    departures
}
