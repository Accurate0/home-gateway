use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::repo::light::{HistorySource, LightSample};

pub struct SampleLightState;

#[async_trait::async_trait]
impl AdhocCronTask for SampleLightState {
    fn name(&self) -> &'static str {
        "sample_light_state"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("*/5 * * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let mut samples = Vec::new();

        for (address, _) in ctx.devices.lights() {
            let Some(state) = ctx.repos.light().get(address).await? else {
                continue;
            };

            samples.push(LightSample {
                address: address.clone(),
                device_id: ctx.devices.id_for_address(address).map(str::to_owned),
                source: HistorySource::Sample,
                event_id: None,
                state,
            });
        }

        ctx.repos.light().record_history_many(&samples).await?;

        Ok(samples.len() as u64)
    }
}
