use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct RefreshFuelWatchSites;

#[async_trait::async_trait]
impl AdhocCronTask for RefreshFuelWatchSites {
    fn name(&self) -> &'static str {
        "refresh_fuelwatch_sites"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("5 */2 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let Some(fuelwatch) = ctx.fuel_watch else {
            tracing::info!("fuelwatch not configured, skipping site refresh");
            return Ok(0);
        };

        let sites = fuelwatch
            .fetch_sites()
            .await
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        if sites.is_empty() {
            tracing::warn!("fuelwatch returned no priced sites, leaving the stored set alone");
            return Ok(0);
        }

        let repo = ctx.repos.fuelwatch();

        let stored = repo.replace_sites(ctx.tx, &sites).await?;
        let appended = repo.append_history(ctx.tx, &sites).await?;

        tracing::info!("stored {stored} fuelwatch sites and appended {appended} history rows");

        Ok(stored)
    }
}
