use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Australia::Perth;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::instrument;

use super::auth::realtime_auth_header;
use super::{Departure, TransperthError, USER_AGENT, normalise_station};

const TIMETABLE_URL: &str = "https://au-journeyplanner.silverrail.io/journeyplannerservice/v2/REST/DataSets/PerthRestricted/Timetable";
const REALTIME_URL: &str = "https://realtime.transperth.info/SJP/Trip";

const MATCH_TOLERANCE_MINUTES: i64 = 2;

#[derive(Debug, Clone, Deserialize)]
pub struct PtaTimetable {
    #[serde(rename = "TimetableTrips", default)]
    pub trips: Vec<PtaTimetableTrip>,
    #[serde(rename = "StopPatterns", default)]
    pub stop_patterns: Vec<PtaStopPattern>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PtaTimetableTrip {
    #[serde(rename = "TripUid")]
    pub trip_uid: String,
    #[serde(rename = "Headsign", default)]
    pub headsign: String,
    #[serde(rename = "StopPatternId", default)]
    pub stop_pattern_id: i64,
    #[serde(rename = "TripStopTimings", default)]
    pub stop_timings: Vec<PtaStopTiming>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PtaStopPattern {
    #[serde(rename = "StopPatternId")]
    pub stop_pattern_id: i64,
    #[serde(rename = "TransitStopUids", default)]
    pub transit_stop_uids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PtaStopTiming {
    #[serde(rename = "DepartTime", default)]
    pub depart_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PtaRealTimeResponse {
    #[serde(rename = "TripStops", default)]
    trip_stops: Vec<PtaTripStop>,
}

#[derive(Debug, Clone, Deserialize)]
struct PtaTripStop {
    #[serde(rename = "RealTimeInfo", default)]
    real_time_info: Option<PtaRealTimeInfo>,
    #[serde(rename = "TransitStop", default)]
    transit_stop: Option<PtaTransitStop>,
}

#[derive(Debug, Clone, Deserialize)]
struct PtaTransitStop {
    #[serde(rename = "Description", default)]
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PtaRealTimeInfo {
    #[serde(rename = "EstimatedDepartureTime", default)]
    estimated_departure_time: Option<String>,
}

#[instrument(skip(client, reference_key))]
pub async fn fetch_timetable(
    client: &ClientWithMiddleware,
    reference_key: &str,
    route_group: &str,
    date: NaiveDate,
) -> Result<PtaTimetable, TransperthError> {
    let date = date.format("%Y-%m-%d").to_string();

    let response = client
        .get(TIMETABLE_URL)
        .header("user-agent", USER_AGENT)
        .query(&[
            ("ApiKey", reference_key),
            ("format", "json"),
            ("Route", route_group),
            ("StartDate", &date),
            ("EndDate", &date),
            ("ReturnNotes", "false"),
        ])
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TransperthError::Status { status, body });
    }

    Ok(response.json().await.map_err(reqwest_middleware::Error::from)?)
}

pub async fn overlay(
    client: &ClientWithMiddleware,
    realtime_key: &str,
    timetable: &PtaTimetable,
    date: NaiveDate,
    departures: &mut [Departure],
) -> Result<(), TransperthError> {
    for departure in departures.iter_mut() {
        if departure.live_departure.is_some() {
            continue;
        }

        let Some(trip_uid) = match_trip(timetable, departure) else {
            continue;
        };

        match fetch_estimate(client, realtime_key, &trip_uid, date, departure).await {
            Ok(Some(live)) => departure.live_departure = Some(live),
            Ok(None) => tracing::debug!("transperth realtime had no estimate for {trip_uid}"),
            Err(error) => tracing::warn!("transperth realtime failed for {trip_uid}: {error}"),
        }
    }

    Ok(())
}

fn match_trip(timetable: &PtaTimetable, departure: &Departure) -> Option<String> {
    let scheduled = departure.scheduled_departure;
    let headsign = normalise_station(&departure.headsign);

    timetable
        .trips
        .iter()
        .filter(|trip| {
            headsign.is_empty() || normalise_station(&trip.headsign) == headsign
        })
        .find_map(|trip| {
            let pattern = timetable
                .stop_patterns
                .iter()
                .find(|pattern| pattern.stop_pattern_id == trip.stop_pattern_id)?;

            if pattern.transit_stop_uids.len() != trip.stop_timings.len() {
                return None;
            }

            let matches = trip.stop_timings.iter().any(|timing| {
                timing
                    .depart_time
                    .as_deref()
                    .and_then(parse_pta_time)
                    .is_some_and(|time| {
                        (time - scheduled).num_minutes().abs() <= MATCH_TOLERANCE_MINUTES
                    })
            });

            matches.then(|| trip.trip_uid.clone())
        })
}

async fn fetch_estimate(
    client: &ClientWithMiddleware,
    realtime_key: &str,
    trip_uid: &str,
    date: NaiveDate,
    departure: &Departure,
) -> Result<Option<DateTime<Utc>>, TransperthError> {
    let body = serde_json::json!({
        "TripUid": trip_uid,
        "TripDate": date.format("%Y-%m-%d").to_string(),
        "IsMappingDataReturned": false,
        "IsRealTimeChecked": true,
        "ReturnNotes": false,
    });

    let response = client
        .post(REALTIME_URL)
        .header("user-agent", USER_AGENT)
        .header(
            "authorization",
            realtime_auth_header(realtime_key, Utc::now()),
        )
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TransperthError::Status { status, body });
    }

    let payload: PtaRealTimeResponse = response
        .json()
        .await
        .map_err(reqwest_middleware::Error::from)?;

    let platform = departure.platform.as_deref().unwrap_or_default();

    Ok(payload
        .trip_stops
        .iter()
        .find(|stop| {
            stop.transit_stop
                .as_ref()
                .is_some_and(|transit| transit.description.contains(platform))
        })
        .or_else(|| payload.trip_stops.first())
        .and_then(|stop| stop.real_time_info.as_ref())
        .and_then(|info| info.estimated_departure_time.as_deref())
        .and_then(parse_pta_time))
}

fn parse_pta_time(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim().trim_start_matches("1.");

    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }

    let naive = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok()?;

    Perth
        .from_local_datetime(&naive)
        .earliest()
        .map(|local| local.with_timezone(&Utc))
}
