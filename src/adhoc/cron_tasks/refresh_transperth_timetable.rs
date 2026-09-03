use std::collections::HashMap;

use crate::actors::integrations::transperth::{TransperthActor, TransperthMessage};
use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::integrations::transperth::{TIMETABLE_KEY, Transperth, gtfs};

const ETAG_METADATA_KEY: &str = "gtfs-etag";
const VERSION_METADATA_KEY: &str = "index-version";
const INDEX_VERSION: &str = "2";

pub struct RefreshTransperthTimetable;

#[async_trait::async_trait]
impl AdhocCronTask for RefreshTransperthTimetable {
    fn name(&self) -> &'static str {
        "refresh_transperth_timetable"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("20 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let Some(settings) = ctx.settings.transperth.clone() else {
            tracing::info!("transperth not configured, skipping timetable refresh");
            return Ok(0);
        };

        let transperth = Transperth::new(&settings)
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        let stored = ctx
            .s3
            .get_object_metadata(TIMETABLE_KEY)
            .await
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?
            .unwrap_or_default();

        let etag = match stored.get(VERSION_METADATA_KEY).map(String::as_str) {
            Some(INDEX_VERSION) => stored.get(ETAG_METADATA_KEY).cloned(),
            _ => {
                tracing::info!("transperth index version changed, rebuilding from gtfs");
                None
            }
        };

        let Some(archive) = transperth
            .download_gtfs(etag.as_deref())
            .await
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?
        else {
            return Ok(0);
        };

        let watched = transperth.watched_station_names();

        let index = gtfs::build_index(&archive.bytes, &watched)
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        let trips = index.trips.len() as u64;

        let payload = serde_json::to_vec(&index)
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        tracing::info!(
            "transperth timetable reduced {} bytes to {} bytes ({trips} trips)",
            archive.bytes.len(),
            payload.len()
        );

        let mut metadata = HashMap::new();
        metadata.insert(VERSION_METADATA_KEY.to_owned(), INDEX_VERSION.to_owned());

        if let Some(etag) = archive.etag {
            metadata.insert(ETAG_METADATA_KEY.to_owned(), etag);
        }

        ctx.s3
            .put_object_with_metadata(TIMETABLE_KEY, &payload, Some("application/json"), &metadata)
            .await
            .map_err(|error| AdhocTaskError::Failed(error.to_string()))?;

        match ractor::registry::where_is(TransperthActor::NAME) {
            Some(actor) => match actor.send_message(TransperthMessage::LoadTimetable) {
                Ok(()) => tracing::info!("transperth actor asked to reload the timetable"),
                Err(error) => tracing::warn!("transperth actor reload failed: {error}"),
            },
            None => tracing::debug!("transperth actor not running, index will load on next start"),
        }

        Ok(trips)
    }
}
