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
    pub trim_device_intent: AdhocCronTaskSettings<RetentionParameters>,
    pub trim_workflow_runs: AdhocCronTaskSettings<RetentionParameters>,
}
