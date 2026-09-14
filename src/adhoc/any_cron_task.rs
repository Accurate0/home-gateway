use crate::settings::adhoc_cron_task_config::AdhocCronTaskConfig;
use crate::settings::adhoc_tasks::AdhocTasksSettings;

use super::context::AdhocTaskContext;
use super::cron_task::AdhocCronTask;
use super::error::AdhocTaskError;

#[async_trait::async_trait]
pub trait AnyAdhocCronTask: Send + Sync {
    fn name(&self) -> &'static str;

    fn config<'a>(&self, tasks: &'a AdhocTasksSettings) -> &'a dyn AdhocCronTaskConfig;

    fn flag(&self) -> Option<&'static str>;

    fn source(&self) -> &'static str;

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError>;
}

#[async_trait::async_trait]
impl<T: AdhocCronTask> AnyAdhocCronTask for T {
    fn name(&self) -> &'static str {
        AdhocCronTask::name(self)
    }

    fn config<'a>(&self, tasks: &'a AdhocTasksSettings) -> &'a dyn AdhocCronTaskConfig {
        self.settings(tasks)
    }

    fn flag(&self) -> Option<&'static str> {
        AdhocCronTask::flag(self)
    }

    fn source(&self) -> &'static str {
        AdhocCronTask::source(self)
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let settings = ctx.settings;
        let parameters = &self.settings(&settings.adhoc.tasks).parameters;

        AdhocCronTask::run(self, ctx, parameters).await
    }
}
