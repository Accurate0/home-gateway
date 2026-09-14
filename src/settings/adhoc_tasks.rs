use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::no_parameters::NoParameters;
use crate::settings::retention_parameters::RetentionParameters;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdhocTasksSettings {
    pub refresh_public_holidays: AdhocCronTaskSettings<NoParameters>,
    pub refresh_transperth_timetable: AdhocCronTaskSettings<NoParameters>,
    pub sample_light_state: AdhocCronTaskSettings<NoParameters>,
    pub trim_derived_door_events: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_device_intent: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_device_metric: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_door_sensor: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_home_assistant_events: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_jellyfin_playback_events: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_light_history: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_robot_vacuum_events: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_smart_switch: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_temperature_sensor: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_workflow_runs: AdhocCronTaskSettings<RetentionParameters>,
}
