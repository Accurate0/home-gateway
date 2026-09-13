use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime};

use super::types::Holiday;

const PUBLIC_HOLIDAY: &str = "Public holiday";

pub fn parse(raw: &str) -> Result<Vec<Holiday>, String> {
    let calendar: Calendar = raw.parse()?;

    Ok(calendar
        .components
        .iter()
        .filter_map(CalendarComponent::as_event)
        .filter_map(|event| {
            let (Some(uid), Some(name), Some(DatePerhapsTime::Date(date))) =
                (event.get_uid(), event.get_summary(), event.get_start())
            else {
                tracing::warn!("skipping a holiday event missing its uid, summary or all-day date");
                return None;
            };

            let (public, regions) = classify(event.get_description().unwrap_or_default());

            Some(Holiday {
                uid: uid.to_owned(),
                date,
                name: name.to_owned(),
                public,
                regions,
            })
        })
        .collect())
}

fn classify(description: &str) -> (bool, Vec<String>) {
    let first = description.lines().next().unwrap_or_default();

    let Some(rest) = first.strip_prefix(PUBLIC_HOLIDAY) else {
        return (false, Vec::new());
    };

    let regions = match rest.strip_prefix(" in ") {
        Some(list) => list
            .split(", ")
            .flat_map(|part| part.split(" and "))
            .map(str::trim)
            .filter(|region| !region.is_empty())
            .map(str::to_owned)
            .collect(),
        None => Vec::new(),
    };

    (true, regions)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    const FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/holidays/basic.ics"
    ));

    fn holidays() -> Vec<Holiday> {
        parse(FIXTURE).expect("expected the fixture to parse")
    }

    #[test]
    fn parses_every_event_in_the_calendar() {
        assert_eq!(holidays().len(), 4);
    }

    #[test]
    fn an_observance_is_not_public() {
        let holidays = holidays();
        let harmony = holidays.iter().find(|h| h.name == "Harmony Day").unwrap();

        assert!(!harmony.public);
        assert!(harmony.regions.is_empty());
        assert_eq!(harmony.date, NaiveDate::from_ymd_opt(2021, 3, 21).unwrap());
    }

    #[test]
    fn a_national_public_holiday_has_no_regions() {
        let holidays = holidays();
        let new_year = holidays
            .iter()
            .find(|h| h.name == "New Year's Day")
            .unwrap();

        assert!(new_year.public);
        assert!(new_year.regions.is_empty());
    }

    #[test]
    fn a_state_holiday_keeps_its_region() {
        let holidays = holidays();
        let kings = holidays
            .iter()
            .find(|h| h.name == "King's Birthday (Western Australia)")
            .unwrap();

        assert!(kings.public);
        assert_eq!(kings.regions, ["Western Australia"]);
        assert_eq!(kings.uid, "20260928_if7u4pp3e20re2mnvpqmr443lo@google.com");
    }

    #[test]
    fn folded_and_escaped_region_lists_are_split() {
        let holidays = holidays();
        let labour = holidays
            .iter()
            .find(|h| h.name == "Labour Day (regional holiday)")
            .unwrap();

        assert_eq!(
            labour.regions,
            [
                "Australian Capital Territory",
                "New South Wales",
                "South Australia"
            ]
        );
    }
}
