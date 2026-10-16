use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

use super::UnraidError;
use super::schema;
use super::types::{
    ArraySnapshot, ArrayState, DiskKind, DiskSnapshot, DiskStatus, ParityStatus, UnraidSnapshot,
};
use crate::serde_lenient::opt_i64;

#[derive(Debug, cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct ArrayQuery {
    pub array: UnraidArray,
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

#[cfg(test)]
mod tests {
    use cynic::{GraphQlResponse, QueryBuilder};

    use super::*;

    const RESPONSE: &str = r#"{
      "data": {
        "array": {
          "state": "STARTED",
          "capacity": { "kilobytes": { "free": "4000000000", "used": "6000000000", "total": "10000000000" } },
          "parityCheckStatus": { "status": "COMPLETED", "progress": 100, "errors": 0, "running": false },
          "parities": [
            { "name": "parity", "type": "PARITY", "status": "DISK_OK", "temp": 34, "size": "11718885324", "fsSize": null, "fsFree": null, "fsUsed": null, "numErrors": 0, "isSpinning": true }
          ],
          "disks": [
            { "name": "disk1", "type": "DATA", "status": "DISK_OK", "temp": 38, "size": 11718885324, "fsSize": "11718885324", "fsFree": "1171888532", "fsUsed": "10546996792", "numErrors": "2", "isSpinning": true },
            { "name": "disk2", "type": "DATA", "status": "DISK_OK", "temp": null, "size": 3907018532, "fsSize": 3907018532, "fsFree": 2000000000, "fsUsed": 1907018532, "numErrors": 0, "isSpinning": false }
          ],
          "caches": [
            { "name": "cache", "type": "CACHE", "status": "DISK_OK", "temp": 41, "size": 976762584, "fsSize": 976762584, "fsFree": 500000000, "fsUsed": 476762584, "numErrors": 0, "isSpinning": true },
            { "name": null, "type": "CACHE", "status": null, "temp": null, "size": null, "fsSize": null, "fsFree": null, "fsUsed": null, "numErrors": null, "isSpinning": null }
          ]
        }
      }
    }"#;

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
