use crate::actors::system::cron::schedule::CronSchedule;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_task_state::AdhocTaskState;

pub trait AdhocCronTaskConfig: Send + Sync {
    fn state(&self) -> AdhocTaskState;

    fn schedule(&self) -> &CronSchedule;
}

impl<P: Send + Sync> AdhocCronTaskConfig for AdhocCronTaskSettings<P> {
    fn state(&self) -> AdhocTaskState {
        self.state
    }

    fn schedule(&self) -> &CronSchedule {
        &self.schedule
    }
}
