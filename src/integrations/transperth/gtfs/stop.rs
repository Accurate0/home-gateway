use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::TransperthError;
use super::{column, required_column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
    pub platform_code: Option<String>,
    pub parent_station: Option<String>,
}

pub(super) fn read(bytes: &[u8]) -> Result<HashMap<String, Stop>, TransperthError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers()?.clone();

    let id_index = required_column(&headers, "stop_id")?;
    let name_index = required_column(&headers, "stop_name")?;
    let platform_index = column(&headers, "platform_code");
    let parent_index = column(&headers, "parent_station");

    let mut stops = HashMap::new();

    for record in reader.records() {
        let record = record?;

        let Some(stop_id) = record.get(id_index).map(str::trim) else {
            continue;
        };

        let platform_code = platform_index
            .and_then(|index| record.get(index))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);

        let parent_station = parent_index
            .and_then(|index| record.get(index))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);

        stops.insert(
            stop_id.to_owned(),
            Stop {
                stop_id: stop_id.to_owned(),
                stop_name: record.get(name_index).unwrap_or_default().trim().to_owned(),
                platform_code,
                parent_station,
            },
        );
    }

    Ok(stops)
}
