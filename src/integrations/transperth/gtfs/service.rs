use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use super::super::TransperthError;
use super::{column, parse_gtfs_date, required_column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub days: [bool; 7],
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub added: HashSet<NaiveDate>,
    pub removed: HashSet<NaiveDate>,
}

impl Service {
    pub fn runs_on(&self, date: NaiveDate) -> bool {
        if self.removed.contains(&date) {
            return false;
        }

        if self.added.contains(&date) {
            return true;
        }

        if date < self.start_date || date > self.end_date {
            return false;
        }

        self.days[date.weekday().num_days_from_monday() as usize]
    }
}

pub(super) fn read(
    calendar: Option<&[u8]>,
    calendar_dates: Option<&[u8]>,
) -> Result<HashMap<String, Service>, TransperthError> {
    let mut services: HashMap<String, Service> = HashMap::new();

    if let Some(bytes) = calendar {
        let mut reader = csv::Reader::from_reader(bytes);
        let headers = reader.headers()?.clone();

        let service_index = required_column(&headers, "service_id")?;
        let day_indexes = [
            column(&headers, "monday"),
            column(&headers, "tuesday"),
            column(&headers, "wednesday"),
            column(&headers, "thursday"),
            column(&headers, "friday"),
            column(&headers, "saturday"),
            column(&headers, "sunday"),
        ];
        let start_index = column(&headers, "start_date");
        let end_index = column(&headers, "end_date");

        for record in reader.records() {
            let record = record?;

            let Some(service_id) = record.get(service_index).map(str::trim) else {
                continue;
            };

            let mut days = [false; 7];
            for (slot, index) in day_indexes.iter().enumerate() {
                days[slot] =
                    index.and_then(|index| record.get(index)).map(str::trim) == Some("1");
            }

            let start_date = start_index
                .and_then(|index| record.get(index))
                .and_then(parse_gtfs_date)
                .unwrap_or(NaiveDate::MIN);

            let end_date = end_index
                .and_then(|index| record.get(index))
                .and_then(parse_gtfs_date)
                .unwrap_or(NaiveDate::MAX);

            services.insert(
                service_id.to_owned(),
                Service {
                    days,
                    start_date,
                    end_date,
                    added: HashSet::new(),
                    removed: HashSet::new(),
                },
            );
        }
    }

    if let Some(bytes) = calendar_dates {
        let mut reader = csv::Reader::from_reader(bytes);
        let headers = reader.headers()?.clone();

        let service_index = required_column(&headers, "service_id")?;
        let date_index = required_column(&headers, "date")?;
        let exception_index = required_column(&headers, "exception_type")?;

        for record in reader.records() {
            let record = record?;

            let Some(service_id) = record.get(service_index).map(str::trim) else {
                continue;
            };

            let Some(date) = record.get(date_index).and_then(parse_gtfs_date) else {
                continue;
            };

            let service = services.entry(service_id.to_owned()).or_insert(Service {
                days: [false; 7],
                start_date: NaiveDate::MAX,
                end_date: NaiveDate::MIN,
                added: HashSet::new(),
                removed: HashSet::new(),
            });

            match record.get(exception_index).map(str::trim) {
                Some("1") => {
                    service.added.insert(date);
                }
                Some("2") => {
                    service.removed.insert(date);
                }
                _ => {}
            }
        }
    }

    Ok(services)
}
