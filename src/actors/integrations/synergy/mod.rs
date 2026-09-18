pub mod lua;

use crate::state::AppState;
use bytes::Bytes;
use chrono::DateTime;
use ractor::Actor;
use std::io::BufRead;

pub enum SynergyMessage {
    NewUpload(Bytes),
}

const HEADER_LINES: usize = 5;

pub struct SynergyActor {
    pub shared_actor_state: AppState,
}

impl SynergyActor {
    pub const NAME: &str = "synergy";

    async fn ingest(&self, csv: Bytes) -> Result<u32, ractor::ActorProcessingErr> {
        let dt_format = "%d/%m/%Y %H:%M %z";
        let mut cursor = std::io::BufReader::new(csv.iter().as_slice());

        for _ in 0..HEADER_LINES {
            let _ = cursor.skip_until(b'\n');
        }

        let mut count = 0;
        let mut skipped = 0;
        let mut rdr = csv::Reader::from_reader(cursor);

        for result in rdr.deserialize() {
            let record: Result<CsvRecord, csv::Error> = result;

            match record {
                Ok(r) => {
                    let energy_used = r.unbilled_usage + r.billed_usage.unwrap_or(0f64);
                    let solar_exported = r.solar_export;
                    let time_unparsed = format!("{} {} +0800", r.date, r.time);
                    let time = DateTime::parse_from_str(&time_unparsed, dt_format)?;

                    self.shared_actor_state
                        .repos
                        .energy()
                        .record(energy_used, solar_exported, time)
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
    #[serde(rename = "Usage not yet billed")]
    unbilled_usage: f64,
    #[serde(rename = "Usage already billed")]
    billed_usage: Option<f64>,
    #[serde(rename = "Generation")]
    solar_export: f64,
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
