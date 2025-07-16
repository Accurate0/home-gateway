use crate::types::SharedActorState;
use chrono::DateTime;
use ractor::Actor;
use std::io::BufRead;
use types::S3BucketEvent;

pub mod types;

pub enum SynergyMessage {
    NewUpload(S3BucketEvent),
}

pub struct SynergyActor {
    pub shared_actor_state: SharedActorState,
}

impl SynergyActor {
    pub const NAME: &str = "synergy";
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
    billed_usage: f64,
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

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SynergyMessage::NewUpload(s3_bucket_event) => {
                let dt_format = "%d/%m/%Y %H:%M %z";
                let bucket = self.shared_actor_state.bucket_accessor.synergy();
                let object_keys = s3_bucket_event.records.iter().map(|r| &r.s3.object.key);
                for object_key in object_keys {
                    let item = bucket.get_object(object_key).await?.into_bytes();
                    let mut cursor = std::io::BufReader::new(item.iter().as_slice());

                    for _ in 0..5 {
                        let _ = cursor.skip_until(b'\n');
                    }

                    let mut rdr = csv::Reader::from_reader(cursor);
                    for result in rdr.deserialize() {
                        let record: Result<CsvRecord, csv::Error> = result;
                        match record {
                            Ok(r) => {
                                let energy_used = r.unbilled_usage + r.billed_usage;
                                let solar_exported = r.solar_export;
                                let time_unparsed = format!("{} {} +0800", r.date, r.time);
                                let time = DateTime::parse_from_str(&time_unparsed, dt_format)?;

                                sqlx::query!(
                                    "INSERT INTO energy_consumption(energy_used, solar_exported, time) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                                    energy_used,
                                    solar_exported,
                                    time
                                )
                                .execute(&self.shared_actor_state.db)
                                .await?;
                            }
                            Err(e) => {
                                tracing::warn!("skipping because of {e}")
                            }
                        }
                    }

                    tracing::info!("processing of {object_key} complete, removing file");
                    bucket.delete_object(object_key).await?;
                }
            }
        }

        Ok(())
    }
}
