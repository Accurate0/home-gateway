use async_graphql::dataloader::Loader;
use chrono::{DateTime, Duration, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub const MAX_HISTORY_WINDOW: Duration = Duration::days(90);

pub type BatteryHistoryKey = (String, DateTime<Utc>);

pub struct DeviceBatteryHistoryDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct BatteryHistoryPoint {
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
    pub time: DateTime<Utc>,
}

struct Row {
    device_id: String,
    battery_voltage: Option<f64>,
    battery_percent: Option<f64>,
    time: DateTime<Utc>,
}

pub fn clamp_since(since: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
    since.max(now - MAX_HISTORY_WINDOW)
}

fn partition(
    keys: &[BatteryHistoryKey],
    rows: Vec<Row>,
) -> HashMap<BatteryHistoryKey, Vec<BatteryHistoryPoint>> {
    let mut map: HashMap<BatteryHistoryKey, Vec<BatteryHistoryPoint>> =
        keys.iter().map(|key| (key.clone(), Vec::new())).collect();

    for row in rows {
        for (key, points) in map.iter_mut() {
            if key.0 == row.device_id && row.time >= key.1 {
                points.push(BatteryHistoryPoint {
                    battery_voltage: row.battery_voltage,
                    battery_percent: row.battery_percent,
                    time: row.time,
                });
            }
        }
    }

    map
}

impl Loader<BatteryHistoryKey> for DeviceBatteryHistoryDataLoader {
    type Value = Vec<BatteryHistoryPoint>;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[BatteryHistoryKey],
    ) -> Result<HashMap<BatteryHistoryKey, Self::Value>, Self::Error> {
        let device_ids = keys.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();

        let Some(earliest) = keys.iter().map(|(_, since)| *since).min() else {
            return Ok(HashMap::new());
        };

        let rows = sqlx::query_as!(
            Row,
            r#"SELECT device_id, battery_voltage, battery_percent, "time"
               FROM device_battery
               WHERE device_id = ANY($1) AND "time" >= $2
               ORDER BY "time" ASC"#,
            &device_ids,
            earliest
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-device-battery-history"))
        .await?;

        Ok(partition(keys, rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&format!("2026-07-28T{hour:02}:00:00Z"))
            .unwrap()
            .with_timezone(&Utc)
    }

    fn row(device_id: &str, hour: u32) -> Row {
        Row {
            device_id: device_id.to_owned(),
            battery_voltage: Some(3.7),
            battery_percent: None,
            time: at(hour),
        }
    }

    #[test]
    fn rows_are_split_per_device() {
        let keys = vec![("a".to_owned(), at(0)), ("b".to_owned(), at(0))];
        let map = partition(&keys, vec![row("a", 1), row("b", 2), row("a", 3)]);

        assert_eq!(map[&keys[0]].len(), 2);
        assert_eq!(map[&keys[1]].len(), 1);
    }

    #[test]
    fn each_key_only_sees_its_own_window() {
        let early = ("a".to_owned(), at(0));
        let late = ("a".to_owned(), at(5));
        let keys = vec![early.clone(), late.clone()];

        let map = partition(&keys, vec![row("a", 1), row("a", 6)]);

        assert_eq!(map[&early].len(), 2);
        assert_eq!(map[&late].len(), 1);
        assert_eq!(map[&late][0].time, at(6));
    }

    #[test]
    fn keys_without_rows_get_an_empty_vec() {
        let keys = vec![("a".to_owned(), at(0)), ("missing".to_owned(), at(0))];
        let map = partition(&keys, vec![row("a", 1)]);

        assert!(map[&keys[1]].is_empty());
    }

    #[test]
    fn since_is_clamped_to_the_max_window() {
        let now = at(12);
        let ancient = now - Duration::days(400);

        assert_eq!(clamp_since(ancient, now), now - MAX_HISTORY_WINDOW);
        assert_eq!(
            clamp_since(now - Duration::days(1), now),
            now - Duration::days(1)
        );
    }
}
