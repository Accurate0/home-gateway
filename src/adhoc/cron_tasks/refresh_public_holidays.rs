use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::integrations::holidays::Holidays;
use crate::settings::HttpClientKind;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::settings::no_parameters::NoParameters;

pub struct RefreshPublicHolidays;

#[async_trait::async_trait]
impl AdhocCronTask for RefreshPublicHolidays {
    type Parameters = NoParameters;

    fn name(&self) -> &'static str {
        "refresh_public_holidays"
    }

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters> {
        &tasks.refresh_public_holidays
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        _parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError> {
        let holidays = Holidays::new(
            &ctx.settings.holidays,
            ctx.settings
                .http
                .clients
                .timeout_for(HttpClientKind::Holidays),
        )
        .map_err(|error| AdhocTaskError::Failed(error.to_string()))?
        .fetch()
        .await
        .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        if holidays.is_empty() {
            tracing::warn!("holiday calendar returned no events, leaving the stored set alone");
            return Ok(0);
        }

        let stored = ctx
            .repos
            .holiday()
            .replace(ctx.tx, &holidays)
            .await
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        tracing::info!("stored {stored} holidays");

        Ok(stored)
    }
}
