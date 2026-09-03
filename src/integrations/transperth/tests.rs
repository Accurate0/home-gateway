use std::io::Write;

use chrono::{TimeDelta, TimeZone, Utc};
use chrono_tz::Australia::Perth;

use crate::settings::TransperthRoute;

use super::gtfs::build_index;
use super::{departures, station_stop_ids};

const STOPS: &str = "location_type, parent_station, stop_id, stop_code, stop_name, stop_desc\n\
0,138,99671,99671,\"Aubin Grove Stn Platform 1\",\"Aubin Grove Stn\"\n\
0,138,99672,99672,\"Aubin Grove Stn Platform 2\",\"Aubin Grove Stn\"\n\
0,64,99601,99601,\"Perth Underground Stn Platform 1\",\"Perth Underground Stn\"\n\
0,,88888,88888,\"Mandurah Stn Platform 1\",\"Mandurah Stn\"\n\
1,,64,64,\"Perth Underground Stn\",\n\
1,,138,138,\"Aubin Grove Stn\",\n";

const ROUTES: &str = "route_id,agency_id,route_short_name,route_long_name,route_desc,route_type\n\
WES-RAI-1829,WES-RAI,,Mandurah Line,,2\n\
BUS-1,BUS,,A Bus Route,,3\n";

const TRIPS: &str = "route_id,service_id,trip_id,direction_id,trip_headsign,shape_id\n\
WES-RAI-1829,WEEKDAY,trip-north,0,Perth Underground,s1\n\
WES-RAI-1829,WEEKDAY,trip-south,1,Mandurah,s2\n\
WES-RAI-1829,WEEKEND,trip-weekend,0,Perth Underground,s3\n\
BUS-1,WEEKDAY,trip-bus,0,Somewhere,s4\n";

const STOP_TIMES: &str = "trip_id,arrival_time,departure_time,stop_id,stop_sequence,pickup_type,drop_off_type\n\
trip-north,08:00:00,08:00:00,99671,1,0,0\n\
trip-north,08:25:00,08:25:00,99601,2,0,0\n\
trip-south,08:10:00,08:10:00,99672,1,0,0\n\
trip-south,08:40:00,08:40:00,88888,2,0,0\n\
trip-weekend,09:00:00,09:00:00,99671,1,0,0\n\
trip-weekend,09:25:00,09:25:00,99601,2,0,0\n\
trip-bus,08:05:00,08:05:00,99671,1,0,0\n\
trip-bus,08:30:00,08:30:00,99601,2,0,0\n";

const CALENDAR: &str = "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\n\
WEEKDAY,1,1,1,1,1,0,0,20260101,20271231\n\
WEEKEND,0,0,0,0,0,1,1,20260101,20271231\n";

fn archive() -> Vec<u8> {
    let mut buffer = Vec::new();

    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();

        for (name, contents) in [
            ("stops.txt", STOPS),
            ("routes.txt", ROUTES),
            ("trips.txt", TRIPS),
            ("stop_times.txt", STOP_TIMES),
            ("calendar.txt", CALENDAR),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(contents.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
    }

    buffer
}

fn watched() -> Vec<String> {
    vec![
        "Aubin Grove Stn".to_owned(),
        "Perth Underground Stn".to_owned(),
    ]
}

fn route() -> TransperthRoute {
    TransperthRoute {
        id: "aubin-grove-to-perth".to_owned(),
        from: "Aubin Grove Stn".to_owned(),
        to: "Perth Underground Stn".to_owned(),
        limit: 5,
        route_group: None,
    }
}

#[test]
fn stations_resolve_through_parent_station() {
    let index = build_index(&archive(), &watched()).unwrap();

    let origin = station_stop_ids(&index, "Aubin Grove Stn");

    assert!(origin.contains("99671"), "platform 1 missing: {origin:?}");
    assert!(origin.contains("99672"), "platform 2 missing: {origin:?}");

    let destination = station_stop_ids(&index, "Perth Underground Stn");

    assert!(destination.contains("99601"));
}

#[test]
fn station_lookup_ignores_stn_suffix_and_case() {
    let index = build_index(&archive(), &watched()).unwrap();

    assert_eq!(
        station_stop_ids(&index, "aubin grove"),
        station_stop_ids(&index, "Aubin Grove Stn")
    );
}

#[test]
fn bus_routes_are_excluded() {
    let index = build_index(&archive(), &watched()).unwrap();

    assert!(!index.trips.contains_key("trip-bus"));
}

#[test]
fn departures_are_direction_correct() {
    let index = build_index(&archive(), &watched()).unwrap();

    let now = Perth
        .with_ymd_and_hms(2026, 9, 2, 7, 0, 0)
        .unwrap()
        .with_timezone(&Utc);

    let found = departures::build(
        &index,
        &route(),
        now,
        TimeDelta::hours(4),
        &[now.with_timezone(&Perth).date_naive()],
    );

    let trips: Vec<&str> = found.iter().map(|d| d.trip_id.as_str()).collect();

    assert_eq!(trips, vec!["trip-north"], "southbound trip must not appear");
    assert_eq!(found[0].line, "Mandurah Line");
    assert_eq!(found[0].platform.as_deref(), None);
}

#[test]
fn services_not_running_today_are_excluded() {
    let index = build_index(&archive(), &watched()).unwrap();

    let saturday = Perth
        .with_ymd_and_hms(2026, 9, 5, 7, 0, 0)
        .unwrap()
        .with_timezone(&Utc);

    let found = departures::build(
        &index,
        &route(),
        saturday,
        TimeDelta::hours(6),
        &[saturday.with_timezone(&Perth).date_naive()],
    );

    let trips: Vec<&str> = found.iter().map(|d| d.trip_id.as_str()).collect();

    assert_eq!(
        trips,
        vec!["trip-weekend"],
        "weekday trip ran on a saturday"
    );
}

#[test]
fn departures_outside_the_horizon_are_excluded() {
    let index = build_index(&archive(), &watched()).unwrap();

    let now = Perth
        .with_ymd_and_hms(2026, 9, 2, 7, 0, 0)
        .unwrap()
        .with_timezone(&Utc);

    let found = departures::build(
        &index,
        &route(),
        now,
        TimeDelta::minutes(30),
        &[now.with_timezone(&Perth).date_naive()],
    );

    assert!(found.is_empty(), "08:00 departure inside a 30m horizon");
}
