use super::{ArraySnapshot, DiskSerials, DiskSnapshot, DiskTemperature};

#[derive(Debug, Clone, PartialEq)]
pub struct UnraidSnapshot {
    pub array: ArraySnapshot,
    pub disks: Vec<DiskSnapshot>,
}

impl UnraidSnapshot {
    pub fn with_temperatures(&self, serials: &DiskSerials, readings: &[DiskTemperature]) -> Self {
        let mut snapshot = self.clone();

        for disk in &mut snapshot.disks {
            let Some(serial) = serials.serial(&disk.name) else {
                continue;
            };

            disk.temperature = readings
                .iter()
                .find(|reading| reading.serial == serial)
                .map(|reading| reading.celsius);
        }

        snapshot
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::integrations::unraid::types::{ArrayState, DiskKind, DiskStatus, ParityStatus};

    fn disk(name: &str, temperature: Option<i32>) -> DiskSnapshot {
        DiskSnapshot {
            name: name.to_owned(),
            kind: DiskKind::Data,
            status: Some(DiskStatus::DiskOk),
            temperature,
            size_kb: None,
            fs_size_kb: None,
            fs_free_kb: None,
            fs_used_kb: None,
            errors: None,
            spinning: Some(true),
        }
    }

    fn snapshot() -> UnraidSnapshot {
        UnraidSnapshot {
            array: ArraySnapshot {
                state: ArrayState::Started,
                kilobytes_free: 0,
                kilobytes_used: 0,
                kilobytes_total: 0,
                parity_status: ParityStatus::NeverRun,
                parity_progress: None,
                parity_errors: None,
                parity_running: false,
            },
            disks: vec![
                disk("disk1", Some(30)),
                disk("parity", Some(29)),
                disk("boot", None),
            ],
        }
    }

    fn serials() -> DiskSerials {
        DiskSerials::new(HashMap::from([
            ("WD-1".to_owned(), "disk1".to_owned()),
            ("ZW-2".to_owned(), "parity".to_owned()),
        ]))
    }

    fn reading(serial: &str, celsius: i32) -> DiskTemperature {
        DiskTemperature {
            serial: serial.to_owned(),
            celsius,
        }
    }

    fn temperature(snapshot: &UnraidSnapshot, name: &str) -> Option<i32> {
        snapshot
            .disks
            .iter()
            .find(|disk| disk.name == name)
            .unwrap()
            .temperature
    }

    #[test]
    fn readings_land_on_the_disk_with_that_serial() {
        let merged =
            snapshot().with_temperatures(&serials(), &[reading("WD-1", 33), reading("ZW-2", 29)]);

        assert_eq!(temperature(&merged, "disk1"), Some(33));
        assert_eq!(temperature(&merged, "parity"), Some(29));
    }

    #[test]
    fn a_disk_missing_from_the_push_has_spun_down() {
        let merged = snapshot().with_temperatures(&serials(), &[reading("WD-1", 30)]);

        assert_eq!(temperature(&merged, "parity"), None);
    }

    #[test]
    fn disks_without_a_known_serial_and_unknown_sensors_are_left_alone() {
        let merged = snapshot().with_temperatures(&serials(), &[reading("OTHER", 50)]);

        assert_eq!(temperature(&merged, "boot"), None);
        assert_eq!(temperature(&merged, "disk1"), None);
    }

    #[test]
    fn an_unchanged_push_is_the_same_snapshot() {
        let merged =
            snapshot().with_temperatures(&serials(), &[reading("WD-1", 30), reading("ZW-2", 29)]);

        assert_eq!(merged, snapshot());
    }
}
