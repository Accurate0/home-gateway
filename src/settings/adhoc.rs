use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AdhocSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub recheck_interval: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub task_timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub cron_jitter: TimeDelta,
    pub batch_size: i64,
    pub tasks: AdhocTasksSettings,
}

impl AdhocSettings {
    pub fn task_timeout(&self) -> Duration {
        self.task_timeout.to_std().unwrap_or_default()
    }

    pub fn cron_jitter(&self) -> Duration {
        self.cron_jitter.to_std().unwrap_or_default()
    }

    pub fn validate(&self) -> Result<(), String> {
        for task in crate::adhoc::cron_registry() {
            if let Err(error) = task.config(&self.tasks).schedule().time_until_next() {
                return Err(format!(
                    "adhoc.tasks.{}.schedule has no next occurrence: {error}",
                    task.name()
                ));
            }
        }

        Ok(())
    }
}
