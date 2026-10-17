use super::{UnraidChange, UnraidMetric, UnraidUpdate};
use crate::integrations::unraid::types::{DiskKind, UnraidNotification, UnraidSnapshot};

pub const ARRAY_SUBJECT: &str = "array";

#[derive(Debug, Clone, PartialEq)]
pub struct UnraidEvent {
    pub subject: String,
    pub kind: Option<DiskKind>,
    pub update: UnraidUpdate,
}

impl UnraidEvent {
    pub fn between(previous: Option<&UnraidSnapshot>, current: &UnraidSnapshot) -> Vec<Self> {
        let mut events = Vec::new();

        if let Some(previous) = previous {
            if previous.array.state != current.array.state {
                events.push(Self::array(UnraidUpdate::Changed {
                    change: UnraidChange::ArrayState,
                    state: current.array.state.to_string(),
                    previous: previous.array.state.to_string(),
                }));
            }

            if previous.array.parity_status != current.array.parity_status {
                events.push(Self::array(UnraidUpdate::Changed {
                    change: UnraidChange::ParityCheck,
                    state: current.array.parity_status.to_string(),
                    previous: previous.array.parity_status.to_string(),
                }));
            }

            for disk in &current.disks {
                let Some(before) = previous
                    .disks
                    .iter()
                    .find(|before| before.name == disk.name)
                else {
                    continue;
                };

                if let (Some(status), Some(was)) = (disk.status, before.status)
                    && status != was
                {
                    events.push(Self {
                        subject: disk.name.clone(),
                        kind: Some(disk.kind),
                        update: UnraidUpdate::Changed {
                            change: UnraidChange::DiskStatus,
                            state: status.to_string(),
                            previous: was.to_string(),
                        },
                    });
                }

                if let (Some(errors), Some(was)) = (disk.errors, before.errors)
                    && errors > was
                {
                    events.push(Self {
                        subject: disk.name.clone(),
                        kind: Some(disk.kind),
                        update: UnraidUpdate::Changed {
                            change: UnraidChange::DiskErrors,
                            state: errors.to_string(),
                            previous: was.to_string(),
                        },
                    });
                }
            }
        }

        for disk in &current.disks {
            if let Some(temperature) = disk.temperature {
                events.push(Self {
                    subject: disk.name.clone(),
                    kind: Some(disk.kind),
                    update: UnraidUpdate::Reading {
                        metric: UnraidMetric::DiskTemperature,
                        value: f64::from(temperature),
                    },
                });
            }

            if let Some(usage) = disk.usage_percent() {
                events.push(Self {
                    subject: disk.name.clone(),
                    kind: Some(disk.kind),
                    update: UnraidUpdate::Reading {
                        metric: UnraidMetric::DiskUsage,
                        value: usage,
                    },
                });
            }
        }

        if let Some(usage) = current.array.usage_percent() {
            events.push(Self::array(UnraidUpdate::Reading {
                metric: UnraidMetric::ArrayUsage,
                value: usage,
            }));
        }

        events
    }

    pub fn notification(notification: UnraidNotification) -> Self {
        Self {
            subject: notification.subject,
            kind: None,
            update: UnraidUpdate::Notification {
                importance: notification.importance,
                title: notification.title,
                description: notification.description,
                link: notification.link,
            },
        }
    }

    fn array(update: UnraidUpdate) -> Self {
        Self {
            subject: ARRAY_SUBJECT.to_owned(),
            kind: None,
            update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::unraid::types::{
        ArraySnapshot, ArrayState, DiskSnapshot, DiskStatus, ParityStatus,
    };

    fn snapshot() -> UnraidSnapshot {
        UnraidSnapshot {
            array: ArraySnapshot {
                state: ArrayState::Started,
                kilobytes_free: 25,
                kilobytes_used: 75,
                kilobytes_total: 100,
                parity_status: ParityStatus::Completed,
                parity_progress: Some(100),
                parity_errors: Some(0),
                parity_running: false,
            },
            disks: vec![DiskSnapshot {
                name: "disk1".to_owned(),
                kind: DiskKind::Data,
                status: Some(DiskStatus::DiskOk),
                temperature: Some(40),
                size_kb: Some(100),
                fs_size_kb: Some(100),
                fs_free_kb: Some(10),
                fs_used_kb: Some(90),
                errors: Some(0),
                spinning: Some(true),
            }],
        }
    }

    fn changes(events: &[UnraidEvent]) -> Vec<(String, UnraidChange, String)> {
        events
            .iter()
            .filter_map(|event| match &event.update {
                UnraidUpdate::Changed { change, state, .. } => {
                    Some((event.subject.clone(), *change, state.clone()))
                }
                UnraidUpdate::Reading { .. } | UnraidUpdate::Notification { .. } => None,
            })
            .collect()
    }

    fn readings(events: &[UnraidEvent]) -> Vec<(String, UnraidMetric, f64)> {
        events
            .iter()
            .filter_map(|event| match &event.update {
                UnraidUpdate::Reading { metric, value } => {
                    Some((event.subject.clone(), *metric, *value))
                }
                UnraidUpdate::Changed { .. } | UnraidUpdate::Notification { .. } => None,
            })
            .collect()
    }

    #[test]
    fn the_first_poll_publishes_only_readings() {
        let events = UnraidEvent::between(None, &snapshot());

        assert!(changes(&events).is_empty());
        assert_eq!(
            readings(&events),
            vec![
                ("disk1".to_owned(), UnraidMetric::DiskTemperature, 40.0),
                ("disk1".to_owned(), UnraidMetric::DiskUsage, 90.0),
                ("array".to_owned(), UnraidMetric::ArrayUsage, 75.0),
            ]
        );
    }

    #[test]
    fn state_status_parity_and_rising_errors_are_changes() {
        let previous = snapshot();
        let mut current = snapshot();

        current.array.state = ArrayState::Stopped;
        current.array.parity_status = ParityStatus::Running;
        current.disks[0].status = Some(DiskStatus::DiskDsbl);
        current.disks[0].errors = Some(3);

        assert_eq!(
            changes(&UnraidEvent::between(Some(&previous), &current)),
            vec![
                (
                    "array".to_owned(),
                    UnraidChange::ArrayState,
                    "stopped".to_owned()
                ),
                (
                    "array".to_owned(),
                    UnraidChange::ParityCheck,
                    "running".to_owned()
                ),
                (
                    "disk1".to_owned(),
                    UnraidChange::DiskStatus,
                    "disk_dsbl".to_owned()
                ),
                ("disk1".to_owned(), UnraidChange::DiskErrors, "3".to_owned()),
            ]
        );
    }

    #[test]
    fn an_unchanged_poll_or_falling_errors_publish_no_changes() {
        let previous = snapshot();
        let mut current = snapshot();

        assert!(changes(&UnraidEvent::between(Some(&previous), &current)).is_empty());

        current.disks[0].errors = None;

        assert!(changes(&UnraidEvent::between(Some(&previous), &current)).is_empty());
    }

    #[test]
    fn a_spun_down_disk_publishes_no_temperature() {
        let mut current = snapshot();

        current.disks[0].temperature = None;

        assert!(
            readings(&UnraidEvent::between(None, &current))
                .iter()
                .all(|(_, metric, _)| *metric != UnraidMetric::DiskTemperature)
        );
    }
}
