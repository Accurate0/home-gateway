use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;

use super::context::AdhocTaskContext;
use super::error::AdhocTaskError;

#[async_trait::async_trait]
pub trait AdhocCronTask: Send + Sync {
    type Parameters: Send + Sync + 'static;

    fn name(&self) -> &'static str;

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters>;

    fn flag(&self) -> Option<&'static str> {
        None
    }

    fn source(&self) -> &'static str;

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError>;
}
