use super::cron_task::AdhocCronTask;

pub mod refresh_transperth_timetable;
pub mod trim_derived_door_events;
pub mod trim_device_metric;
pub mod trim_door_sensor;
pub mod trim_home_assistant_events;
pub mod trim_jellyfin_playback_events;
pub mod trim_robot_vacuum_events;
pub mod trim_smart_switch;
pub mod trim_temperature_sensor;
pub mod trim_unifi_clients;
pub mod trim_workflow_runs;

pub fn all() -> Vec<&'static dyn AdhocCronTask> {
    vec![
        &refresh_transperth_timetable::RefreshTransperthTimetable,
        &trim_derived_door_events::TrimDerivedDoorEvents,
        &trim_device_metric::TrimDeviceMetric,
        &trim_door_sensor::TrimDoorSensor,
        &trim_home_assistant_events::TrimHomeAssistantEvents,
        &trim_jellyfin_playback_events::TrimJellyfinPlaybackEvents,
        &trim_robot_vacuum_events::TrimRobotVacuumEvents,
        &trim_smart_switch::TrimSmartSwitch,
        &trim_temperature_sensor::TrimTemperatureSensor,
        &trim_unifi_clients::TrimUnifiClients,
        &trim_workflow_runs::TrimWorkflowRuns,
    ]
}
