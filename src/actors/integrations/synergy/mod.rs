pub mod lua;

use crate::state::AppState;
use bytes::Bytes;
use chrono::{DateTime, FixedOffset};
use ractor::Actor;

pub enum SynergyMessage {
    NewUpload(Bytes),
}

pub struct SynergyActor {
    pub shared_actor_state: AppState,
}

impl SynergyActor {
    pub const NAME: &str = "synergy";

    async fn ingest(&self, csv: Bytes) -> Result<u32, ractor::ActorProcessingErr> {
        let mut count = 0;
        let mut skipped = 0;
        let mut rdr = csv::Reader::from_reader(csv.as_ref());

        for result in rdr.deserialize() {
            let record: Result<CsvRecord, csv::Error> = result;

            match record {
                Ok(r) => {
                    self.shared_actor_state
                        .repos
                        .energy()
                        .record(r.usage, r.solar_export, r.time()?)
                        .await?;

                    tracing::info!("record: {count} added");
                    count += 1;
                }
                Err(e) => {
                    tracing::info!("record: {count} skipped");
                    tracing::warn!("skipping because of {e}");
                    count += 1;
                    skipped += 1;
                }
            }
        }

        tracing::info!("processing completed: {count} records, {skipped} skipped");

        Ok(skipped)
    }
}

#[derive(Debug, serde::Deserialize)]
struct CsvRecord {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "ANYTIME (KWH)")]
    usage: f64,
    #[serde(rename = "Solar export (Units)")]
    solar_export: f64,
}

impl CsvRecord {
    fn time(&self) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        DateTime::parse_from_str(
            &format!("{} {} +0800", self.date, self.time),
            "%d/%m/%Y %H:%M %z",
        )
    }
}

impl Actor for SynergyActor {
    type Msg = SynergyMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(
        parent = None,
        name = "actor.synergy",
        skip(self, _myself, message, _state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SynergyMessage::NewUpload(csv) => {
                let started = std::time::Instant::now();

                match self.ingest(csv).await {
                    Ok(skipped) => {
                        let outcome = if skipped > 0 {
                            "partial_error"
                        } else {
                            "success"
                        };

                        crate::metrics::record_integration_poll(
                            "synergy",
                            outcome,
                            started.elapsed(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("error ingesting the synergy upload: {e}");
                        crate::tracing_context::record_current_error(&e.to_string());
                        crate::metrics::record_integration_poll(
                            "synergy",
                            "error",
                            started.elapsed(),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPORT: &str = "Date,Time,ANYTIME (KWH),Solar export (Units),Billing Status\n\
        26/06/2026,00:00,0.190,0.000,Billed\n\
        23/09/2026,23:30,0.185,1.250,Not yet billed\n";

    fn records() -> Vec<CsvRecord> {
        csv::Reader::from_reader(EXPORT.as_bytes())
            .deserialize()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn an_interval_export_reads_every_row() {
        let records = records();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].usage, 0.190);
        assert_eq!(records[1].solar_export, 1.250);
    }

    #[test]
    fn an_interval_time_is_perth_local() {
        let time = records()[1].time().unwrap();

        assert_eq!(time.to_rfc3339(), "2026-09-23T23:30:00+08:00");
    }
}
