use crate::actors::system::cron::schedule::CronSchedule;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::enabled_state::EnabledState;

pub trait AdhocCronTaskConfig: Send + Sync {
    fn state(&self) -> EnabledState;

    fn schedule(&self) -> &CronSchedule;
}

impl<P: Send + Sync> AdhocCronTaskConfig for AdhocCronTaskSettings<P> {
    fn state(&self) -> EnabledState {
        self.state
    }

    fn schedule(&self) -> &CronSchedule {
        &self.schedule
    }
}
