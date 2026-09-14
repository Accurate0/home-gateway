use crate::actors::system::cron::schedule::CronSchedule;
use crate::settings::adhoc_task_state::AdhocTaskState;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdhocCronTaskSettings<P> {
    pub state: AdhocTaskState,
    pub schedule: CronSchedule,
    pub parameters: P,
}
