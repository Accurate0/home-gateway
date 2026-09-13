use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::integrations::holidays::Holidays;
use crate::settings::HttpClientKind;

pub struct RefreshPublicHolidays;

#[async_trait::async_trait]
impl AdhocCronTask for RefreshPublicHolidays {
    fn name(&self) -> &'static str {
        "refresh_public_holidays"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("0 4 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
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
