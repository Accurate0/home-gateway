use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::super::TransperthError;
use super::{Trip, column, parse_gtfs_time, required_column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopTime {
    pub stop_id: String,
    pub stop_sequence: u32,
    pub departure_seconds: i64,
    pub pickup_allowed: bool,
    pub dropoff_allowed: bool,
}

pub(super) fn read(
    bytes: &[u8],
    rail_trips: &HashMap<String, Trip>,
    station_ids: &HashSet<String>,
) -> Result<HashMap<String, Vec<StopTime>>, TransperthError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers()?.clone();

    let trip_index = required_column(&headers, "trip_id")?;
    let stop_index = required_column(&headers, "stop_id")?;
    let sequence_index = required_column(&headers, "stop_sequence")?;
    let departure_index = required_column(&headers, "departure_time")?;
    let pickup_index = column(&headers, "pickup_type");
    let dropoff_index = column(&headers, "drop_off_type");

    let mut by_trip: HashMap<String, Vec<StopTime>> = HashMap::new();

    for record in reader.records() {
        let record = record?;

        let Some(trip_id) = record.get(trip_index).map(str::trim) else {
            continue;
        };

        if !rail_trips.contains_key(trip_id) {
            continue;
        }

        let Some(stop_id) = record.get(stop_index).map(str::trim) else {
            continue;
        };

        let Some(stop_sequence) = record
            .get(sequence_index)
            .and_then(|value| value.trim().parse::<u32>().ok())
        else {
            continue;
        };

        let departure_seconds = record
            .get(departure_index)
            .and_then(parse_gtfs_time)
            .unwrap_or_default();

        let pickup_allowed = pickup_index
            .and_then(|index| record.get(index))
            .map(str::trim)
            != Some("1");

        let dropoff_allowed = dropoff_index
            .and_then(|index| record.get(index))
            .map(str::trim)
            != Some("1");

        by_trip
            .entry(trip_id.to_owned())
            .or_default()
            .push(StopTime {
                stop_id: stop_id.to_owned(),
                stop_sequence,
                departure_seconds,
                pickup_allowed,
                dropoff_allowed,
            });
    }

    by_trip.retain(|_, stop_times| {
        stop_times
            .iter()
            .any(|stop_time| station_ids.contains(&stop_time.stop_id))
    });

    for stop_times in by_trip.values_mut() {
        stop_times.sort_by_key(|stop_time| stop_time.stop_sequence);
    }

    Ok(by_trip)
}
