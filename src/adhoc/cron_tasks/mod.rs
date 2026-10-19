use super::any_cron_task::AnyAdhocCronTask;

pub mod refresh_public_holidays;
pub mod refresh_transperth_timetable;
pub mod sample_light_state;
pub mod trim_device_intent;
pub mod trim_workflow_runs;

pub fn all() -> Vec<&'static dyn AnyAdhocCronTask> {
    vec![
        &refresh_public_holidays::RefreshPublicHolidays,
        &refresh_transperth_timetable::RefreshTransperthTimetable,
        &sample_light_state::SampleLightState,
        &trim_device_intent::TrimDeviceIntent,
        &trim_workflow_runs::TrimWorkflowRuns,
    ]
}
