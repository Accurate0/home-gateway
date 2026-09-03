use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::TransperthError;
use super::{RAIL_ROUTE_TYPE, column, required_column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub trip_id: String,
    pub service_id: String,
    pub line: String,
    pub headsign: String,
}

pub(super) fn read_rail_routes(bytes: &[u8]) -> Result<HashMap<String, String>, TransperthError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers()?.clone();

    let id_index = required_column(&headers, "route_id")?;
    let type_index = required_column(&headers, "route_type")?;
    let long_name_index = column(&headers, "route_long_name");
    let short_name_index = column(&headers, "route_short_name");

    let mut routes = HashMap::new();

    for record in reader.records() {
        let record = record?;

        if record.get(type_index).map(str::trim) != Some(RAIL_ROUTE_TYPE) {
            continue;
        }

        let Some(route_id) = record.get(id_index).map(str::trim) else {
            continue;
        };

        let name = long_name_index
            .and_then(|index| record.get(index))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                short_name_index
                    .and_then(|index| record.get(index))
                    .map(str::trim)
            })
            .unwrap_or_default()
            .to_owned();

        routes.insert(route_id.to_owned(), name);
    }

    Ok(routes)
}

pub(super) fn read(
    bytes: &[u8],
    rail_routes: &HashMap<String, String>,
) -> Result<HashMap<String, Trip>, TransperthError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers()?.clone();

    let trip_index = required_column(&headers, "trip_id")?;
    let route_index = required_column(&headers, "route_id")?;
    let service_index = required_column(&headers, "service_id")?;
    let headsign_index = column(&headers, "trip_headsign");

    let mut trips = HashMap::new();

    for record in reader.records() {
        let record = record?;

        let Some(line) = record
            .get(route_index)
            .map(str::trim)
            .and_then(|route_id| rail_routes.get(route_id))
        else {
            continue;
        };

        let Some(trip_id) = record.get(trip_index).map(str::trim) else {
            continue;
        };

        let headsign = headsign_index
            .and_then(|index| record.get(index))
            .unwrap_or_default()
            .trim()
            .to_owned();

        trips.insert(
            trip_id.to_owned(),
            Trip {
                trip_id: trip_id.to_owned(),
                service_id: record.get(service_index).unwrap_or_default().trim().to_owned(),
                line: line.clone(),
                headsign,
            },
        );
    }

    Ok(trips)
}
