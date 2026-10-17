use std::collections::HashMap;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

use super::UnraidError;
use super::schema;
use super::types::{
    ArraySnapshot, ArrayState, DiskKind, DiskSerials, DiskSnapshot, DiskStatus, DiskTemperature,
    NotificationImportance, ParityStatus, SensorType, UnraidNotification, UnraidPoll,
    UnraidSnapshot,
};
use crate::serde_lenient::opt_i64;

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct ArrayQuery {
    pub array: UnraidArray,
    pub disks: Vec<PhysicalDisk>,
}

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Disk")]
pub struct PhysicalDisk {
    pub device: String,
    pub serial_num: String,
}

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Subscription")]
pub struct TemperatureSubscription {
    pub system_metrics_temperature: Option<TemperatureMetrics>,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct TemperatureMetrics {
    pub sensors: Vec<TemperatureSensor>,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct TemperatureSensor {
    pub id: PrefixedId,
    #[cynic(rename = "type")]
    pub kind: SensorType,
    pub current: TemperatureReading,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct TemperatureReading {
    pub value: f64,
}

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Subscription")]
pub struct NotificationSubscription {
    pub notification_added: Notification,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct UnreadNotificationsVariables {
    pub limit: i32,
}

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "UnreadNotificationsVariables")]
pub struct UnreadNotificationsQuery {
    pub notifications: UnreadNotifications,
}

#[derive(Debug, cynic::QueryFragment)]
#[cynic(
    graphql_type = "Notifications",
    variables = "UnreadNotificationsVariables"
)]
pub struct UnreadNotifications {
    #[arguments(filter: { type: "UNREAD", offset: 0, limit: $limit })]
    pub list: Vec<Notification>,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct Notification {
    pub id: PrefixedId,
    pub title: String,
    pub subject: String,
    pub description: String,
    pub importance: NotificationImportance,
    pub link: Option<String>,
}

#[derive(Debug, Clone, cynic::Scalar)]
#[cynic(graphql_type = "PrefixedID")]
pub struct PrefixedId(pub String);

impl From<Notification> for UnraidNotification {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id.0,
            title: notification.title,
            subject: notification.subject,
            description: notification.description,
            importance: notification.importance.into(),
            link: notification.link.filter(|link| !link.trim().is_empty()),
        }
    }
}

#[derive(Debug, cynic::QueryFragment)]
pub struct UnraidArray {
    pub state: ArrayState,
    pub capacity: ArrayCapacity,
    pub parity_check_status: ParityCheck,
    pub parities: Vec<ArrayDisk>,
    pub disks: Vec<ArrayDisk>,
    pub caches: Vec<ArrayDisk>,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ArrayCapacity {
    pub kilobytes: Capacity,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct Capacity {
    pub free: String,
    pub used: String,
    pub total: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ParityCheck {
    pub status: ParityStatus,
    pub progress: Option<i32>,
    pub errors: Option<i32>,
    pub running: Option<bool>,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ArrayDisk {
    pub name: Option<String>,
    pub device: Option<String>,
    #[cynic(rename = "type")]
    pub kind: DiskKind,
    pub status: Option<DiskStatus>,
    pub temp: Option<i32>,
    pub size: Option<BigInt>,
    pub fs_size: Option<BigInt>,
    pub fs_free: Option<BigInt>,
    pub fs_used: Option<BigInt>,
    pub num_errors: Option<BigInt>,
    pub is_spinning: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BigInt(pub i64);

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        opt_i64(deserializer)?
            .map(BigInt)
            .ok_or_else(|| D::Error::custom("expected a whole number"))
    }
}

cynic::impl_scalar!(BigInt, schema::BigInt);

fn kilobytes(field: &'static str, value: &str) -> Result<i64, UnraidError> {
    value.trim().parse().map_err(|_| UnraidError::Unexpected {
        field,
        value: value.to_owned(),
    })
}

impl ArrayDisk {
    fn into_snapshot(self) -> Option<DiskSnapshot> {
        Some(DiskSnapshot {
            name: self.name?,
            kind: self.kind,
            status: self.status,
            temperature: self.temp,
            size_kb: self.size.map(|size| size.0),
            fs_size_kb: self.fs_size.map(|size| size.0),
            fs_free_kb: self.fs_free.map(|size| size.0),
            fs_used_kb: self.fs_used.map(|size| size.0),
            errors: self.num_errors.map(|errors| errors.0),
            spinning: self.is_spinning,
        })
    }
}

impl TryFrom<UnraidArray> for UnraidSnapshot {
    type Error = UnraidError;

    fn try_from(array: UnraidArray) -> Result<Self, Self::Error> {
        let UnraidArray {
            state,
            capacity,
            parity_check_status,
            parities,
            disks,
            caches,
        } = array;

        let capacity = capacity.kilobytes;

        let array = ArraySnapshot {
            state,
            kilobytes_free: kilobytes("capacity.kilobytes.free", &capacity.free)?,
            kilobytes_used: kilobytes("capacity.kilobytes.used", &capacity.used)?,
            kilobytes_total: kilobytes("capacity.kilobytes.total", &capacity.total)?,
            parity_status: parity_check_status.status,
            parity_progress: parity_check_status.progress.map(i64::from),
            parity_errors: parity_check_status.errors.map(i64::from),
            parity_running: parity_check_status.running.unwrap_or(false),
        };

        let disks = parities
            .into_iter()
            .chain(disks)
            .chain(caches)
            .filter_map(ArrayDisk::into_snapshot)
            .collect();

        Ok(UnraidSnapshot { array, disks })
    }
}

impl TryFrom<ArrayQuery> for UnraidPoll {
    type Error = UnraidError;

    fn try_from(query: ArrayQuery) -> Result<Self, Self::Error> {
        let ArrayQuery { array, disks } = query;

        let names = array
            .parities
            .iter()
            .chain(&array.disks)
            .chain(&array.caches)
            .filter_map(|disk| Some((disk.device.clone()?, disk.name.clone()?)))
            .collect::<HashMap<_, _>>();

        let serials = disks
            .into_iter()
            .filter(|disk| !disk.serial_num.trim().is_empty())
            .filter_map(|disk| {
                let name = names.get(&disk.device)?.clone();

                Some((disk.serial_num, name))
            })
            .collect();

        Ok(UnraidPoll {
            snapshot: array.try_into()?,
            serials: DiskSerials::new(serials),
        })
    }
}

impl TemperatureMetrics {
    pub fn disk_temperatures(self) -> Vec<DiskTemperature> {
        self.sensors
            .into_iter()
            .filter(|sensor| matches!(sensor.kind, SensorType::Disk | SensorType::Nvme))
            .map(|sensor| {
                let id = sensor.id.0;

                let serial = match id.split_once(':') {
                    Some((_, serial)) => serial.to_owned(),
                    None => id,
                };

                DiskTemperature {
                    serial,
                    celsius: sensor.current.value.round() as i32,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use cynic::{GraphQlResponse, QueryBuilder, SubscriptionBuilder};

    use super::*;
    use crate::integrations::unraid::types::UnraidImportance;

    const RESPONSE: &str = r#"{
      "data": {
        "array": {
          "state": "STARTED",
          "capacity": { "kilobytes": { "free": "4000000000", "used": "6000000000", "total": "10000000000" } },
          "parityCheckStatus": { "status": "COMPLETED", "progress": 100, "errors": 0, "running": false },
          "parities": [
            { "name": "parity", "device": "sdb", "type": "PARITY", "status": "DISK_OK", "temp": 34, "size": "11718885324", "fsSize": null, "fsFree": null, "fsUsed": null, "numErrors": 0, "isSpinning": true }
          ],
          "disks": [
            { "name": "disk1", "device": "sda", "type": "DATA", "status": "DISK_OK", "temp": 38, "size": 11718885324, "fsSize": "11718885324", "fsFree": "1171888532", "fsUsed": "10546996792", "numErrors": "2", "isSpinning": true },
            { "name": "disk2", "device": null, "type": "DATA", "status": "DISK_OK", "temp": null, "size": 3907018532, "fsSize": 3907018532, "fsFree": 2000000000, "fsUsed": 1907018532, "numErrors": 0, "isSpinning": false }
          ],
          "caches": [
            { "name": "cache", "device": "nvme0n1", "type": "CACHE", "status": "DISK_OK", "temp": 41, "size": 976762584, "fsSize": 976762584, "fsFree": 500000000, "fsUsed": 476762584, "numErrors": 0, "isSpinning": true },
            { "name": null, "device": null, "type": "CACHE", "status": null, "temp": null, "size": null, "fsSize": null, "fsFree": null, "fsUsed": null, "numErrors": null, "isSpinning": null }
          ]
        },
        "disks": [
          { "device": "sda", "serialNum": "WD-WX72A70242TP" },
          { "device": "sdb", "serialNum": "ZW6464KJ" },
          { "device": "sdc", "serialNum": "" },
          { "device": "sdz", "serialNum": "UNASSIGNED" }
        ]
      }
    }"#;

    const TEMPERATURES: &str = r#"{
      "data": {
        "systemMetricsTemperature": {
          "sensors": [
            { "id": "disk:WD-WX72A70242TP", "type": "DISK", "current": { "value": 30 } },
            { "id": "disk:ZW6464KJ", "type": "DISK", "current": { "value": 28.6 } },
            { "id": "cpu:package", "type": "CPU_PACKAGE", "current": { "value": 55 } }
          ]
        }
      }
    }"#;

    #[test]
    fn the_poll_maps_serials_to_array_disk_names() {
        let response: GraphQlResponse<ArrayQuery> = serde_json::from_str(RESPONSE).unwrap();
        let poll = UnraidPoll::try_from(response.data.unwrap()).unwrap();

        assert_eq!(poll.serials.name("WD-WX72A70242TP"), Some("disk1"));
        assert_eq!(poll.serials.name("ZW6464KJ"), Some("parity"));
        assert_eq!(poll.serials.name("UNASSIGNED"), None);
        assert_eq!(poll.serials.serial("cache"), None);
        assert_eq!(poll.snapshot.disks.len(), 4);
    }

    #[test]
    fn disk_sensors_become_temperatures_by_serial() {
        let response: GraphQlResponse<TemperatureSubscription> =
            serde_json::from_str(TEMPERATURES).unwrap();

        let temperatures = response
            .data
            .unwrap()
            .system_metrics_temperature
            .unwrap()
            .disk_temperatures();

        assert_eq!(
            temperatures,
            vec![
                DiskTemperature {
                    serial: "WD-WX72A70242TP".to_owned(),
                    celsius: 30,
                },
                DiskTemperature {
                    serial: "ZW6464KJ".to_owned(),
                    celsius: 29,
                },
            ]
        );
    }

    #[test]
    fn an_added_notification_parses() {
        let response: GraphQlResponse<NotificationSubscription> = serde_json::from_str(
            r#"{
              "data": {
                "notificationAdded": {
                  "id": "abc:1760000000.notify",
                  "title": "Unraid Parity check",
                  "subject": "Notice [TOWER] - Parity check finished (0 errors)",
                  "description": "Duration: 6 hours, 2 minutes",
                  "importance": "INFO",
                  "link": ""
                }
              }
            }"#,
        )
        .unwrap();

        let notification = UnraidNotification::from(response.data.unwrap().notification_added);

        assert_eq!(notification.importance, UnraidImportance::Info);
        assert_eq!(notification.title, "Unraid Parity check");
        assert_eq!(notification.link, None);
    }

    #[test]
    fn the_backfill_lists_unread_notifications() {
        let operation = UnreadNotificationsQuery::build(UnreadNotificationsVariables { limit: 50 });

        assert!(operation.query.contains("UNREAD"), "{}", operation.query);
        assert!(operation.query.contains("$limit"), "{}", operation.query);

        let response: GraphQlResponse<UnreadNotificationsQuery> = serde_json::from_str(
            r#"{
              "data": {
                "notifications": {
                  "list": [
                    { "id": "n2", "title": "Unraid Status", "subject": "Array started", "description": "", "importance": "ALERT", "link": "/Main" }
                  ]
                }
              }
            }"#,
        )
        .unwrap();

        let list = response.data.unwrap().notifications.list;

        assert_eq!(list.len(), 1);
        assert_eq!(
            UnraidNotification::from(list.into_iter().next().unwrap()).importance,
            UnraidImportance::Alert
        );
    }

    #[test]
    fn the_subscription_asks_for_sensor_ids_and_values() {
        let subscription = TemperatureSubscription::build(());
        let query = subscription.query();

        assert!(query.starts_with("subscription"), "{query}");

        for field in ["systemMetricsTemperature", "sensors", "id", "type", "value"] {
            assert!(query.contains(field), "missing {field} in {query}");
        }
    }

    fn snapshot() -> UnraidSnapshot {
        let response: GraphQlResponse<ArrayQuery> =
            serde_json::from_str(RESPONSE).expect("expected the sample to parse");

        response
            .data
            .expect("expected data")
            .array
            .try_into()
            .expect("expected a snapshot")
    }

    #[test]
    fn the_query_asks_for_the_array_and_every_disk_group() {
        let query = ArrayQuery::build(()).query;

        for field in [
            "parityCheckStatus",
            "parities",
            "disks",
            "caches",
            "fsUsed",
            "numErrors",
        ] {
            assert!(query.contains(field), "missing {field} in {query}");
        }
    }

    #[test]
    fn the_array_and_its_capacity_parse() {
        let array = snapshot().array;

        assert_eq!(array.state, ArrayState::Started);
        assert_eq!(array.kilobytes_total, 10_000_000_000);
        assert_eq!(array.parity_status, ParityStatus::Completed);
        assert_eq!(array.usage_percent(), Some(60.0));
    }

    #[test]
    fn big_ints_arrive_as_numbers_or_strings() {
        let disks = snapshot().disks;
        let disk1 = disks.iter().find(|disk| disk.name == "disk1").unwrap();

        assert_eq!(disk1.kind, DiskKind::Data);
        assert_eq!(disk1.status, Some(DiskStatus::DiskOk));
        assert_eq!(disk1.size_kb, Some(11_718_885_324));
        assert_eq!(disk1.errors, Some(2));
        assert_eq!(disk1.usage_percent().map(f64::round), Some(90.0));
    }

    #[test]
    fn a_spun_down_disk_has_no_temperature_and_unnamed_slots_are_dropped() {
        let disks = snapshot().disks;

        assert_eq!(disks.len(), 4);
        assert_eq!(
            disks
                .iter()
                .find(|disk| disk.name == "disk2")
                .unwrap()
                .temperature,
            None
        );
        assert_eq!(
            disks
                .iter()
                .find(|disk| disk.name == "parity")
                .unwrap()
                .usage_percent(),
            None
        );
    }

    #[test]
    fn an_unparseable_capacity_is_an_error() {
        let response = RESPONSE.replace("\"10000000000\"", "\"lots\"");
        let response: GraphQlResponse<ArrayQuery> = serde_json::from_str(&response).unwrap();
        let error = UnraidSnapshot::try_from(response.data.unwrap().array).unwrap_err();

        assert!(error.to_string().contains("lots"), "{error}");
    }
}
