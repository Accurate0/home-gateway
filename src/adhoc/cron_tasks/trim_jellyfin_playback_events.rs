use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct TrimJellyfinPlaybackEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimJellyfinPlaybackEvents {
    fn name(&self) -> &'static str {
        "trim_jellyfin_playback_events"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("35 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('jellyfin_playback_events', older_than => now() - INTERVAL '180 days')::text"
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
