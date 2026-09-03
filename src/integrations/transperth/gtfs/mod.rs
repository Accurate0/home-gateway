use std::collections::HashSet;
use std::io::{Cursor, Read};

use chrono::{NaiveDate, NaiveDateTime, TimeDelta};

use super::{TransperthError, normalise_station};

mod service;
mod stop;
mod stop_time;
mod timetable_index;
mod trip;

pub use service::Service;
pub use stop::Stop;
pub use stop_time::StopTime;
pub use timetable_index::TimetableIndex;
pub use trip::Trip;

pub(crate) const RAIL_ROUTE_TYPE: &str = "2";

pub fn parse_gtfs_time(value: &str) -> Option<i64> {
    let mut parts = value.trim().split(':');
    let hours: i64 = parts.next()?.trim().parse().ok()?;
    let minutes: i64 = parts.next()?.trim().parse().ok()?;
    let seconds: i64 = parts.next().unwrap_or("0").trim().parse().ok()?;

    Some(hours * 3600 + minutes * 60 + seconds)
}

pub fn parse_gtfs_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y%m%d").ok()
}

pub fn service_datetime(date: NaiveDate, offset_seconds: i64) -> Option<NaiveDateTime> {
    date.and_hms_opt(0, 0, 0)?
        .checked_add_signed(TimeDelta::seconds(offset_seconds))
}

pub(crate) fn column(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header.trim() == name)
}

pub(crate) fn required_column(
    headers: &csv::StringRecord,
    name: &'static str,
) -> Result<usize, TransperthError> {
    column(headers, name).ok_or(TransperthError::MissingGtfsColumn(name))
}

fn read_entry(archive_bytes: &[u8], name: &str) -> Result<Option<Vec<u8>>, TransperthError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(archive_bytes))?;

    let mut file = match archive.by_name(name) {
        Ok(file) => file,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(Some(buffer))
}

fn required_entry(archive_bytes: &[u8], name: &str) -> Result<Vec<u8>, TransperthError> {
    read_entry(archive_bytes, name)?
        .ok_or_else(|| TransperthError::MissingGtfsEntry(name.to_owned()))
}

pub fn build_index(
    archive_bytes: &[u8],
    watched: &[String],
) -> Result<TimetableIndex, TransperthError> {
    let all_stops = stop::read(&required_entry(archive_bytes, "stops.txt")?)?;

    let mut stations: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for name in watched {
        let wanted = normalise_station(name);

        let ids: Vec<String> = all_stops
            .values()
            .filter(|stop| {
                normalise_station(&stop.stop_name) == wanted
                    || stop
                        .parent_station
                        .as_ref()
                        .and_then(|parent| all_stops.get(parent))
                        .is_some_and(|parent| normalise_station(&parent.stop_name) == wanted)
            })
            .map(|stop| stop.stop_id.clone())
            .collect();

        if ids.is_empty() {
            tracing::warn!("transperth station {name} matched no gtfs stops");
        }

        stations.insert(wanted, ids);
    }

    let station_ids: HashSet<String> = stations.values().flatten().cloned().collect();

    if station_ids.is_empty() {
        return Err(TransperthError::NoWatchedStops);
    }

    let rail_routes = trip::read_rail_routes(&required_entry(archive_bytes, "routes.txt")?)?;
    let rail_trips = trip::read(&required_entry(archive_bytes, "trips.txt")?, &rail_routes)?;

    let stop_times = stop_time::read(
        &required_entry(archive_bytes, "stop_times.txt")?,
        &rail_trips,
        &station_ids,
    )?;

    let mut services = service::read(
        read_entry(archive_bytes, "calendar.txt")?.as_deref(),
        read_entry(archive_bytes, "calendar_dates.txt")?.as_deref(),
    )?;

    let trips: std::collections::HashMap<String, Trip> = rail_trips
        .into_iter()
        .filter(|(trip_id, _)| stop_times.contains_key(trip_id))
        .collect();

    let referenced_stops: HashSet<String> = stop_times
        .values()
        .flatten()
        .map(|stop_time| stop_time.stop_id.clone())
        .collect();

    let stops = all_stops
        .into_iter()
        .filter(|(stop_id, _)| referenced_stops.contains(stop_id))
        .collect();

    services.retain(|service_id, _| trips.values().any(|trip| &trip.service_id == service_id));

    Ok(TimetableIndex {
        generated_at: chrono::Utc::now(),
        stations,
        stops,
        trips,
        stop_times,
        services,
    })
}
