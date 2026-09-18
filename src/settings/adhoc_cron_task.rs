use crate::actors::system::cron::schedule::CronSchedule;
use crate::settings::enabled_state::EnabledState;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdhocCronTaskSettings<P> {
    pub state: EnabledState,
    pub schedule: CronSchedule,
    pub parameters: P,
}
