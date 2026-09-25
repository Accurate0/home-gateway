pub mod interval;
pub mod lua;

use crate::lua::LuaCallContext;
use crate::state::AppState;
use bytes::Bytes;
use interval::EnergyInterval;
use ractor::Actor;
use uuid::Uuid;

pub enum SynergyMessage {
    NewUpload(Bytes),
}

pub struct SynergyActor {
    pub shared_actor_state: AppState,
}

impl SynergyActor {
    pub const NAME: &str = "synergy";

    async fn ingest(&self, csv: Bytes) -> Result<usize, ractor::ActorProcessingErr> {
        let state = &self.shared_actor_state;
        let csv = String::from_utf8(csv.to_vec())?;
        let parser = &state.settings.integrations.synergy.parser;
        let cx = LuaCallContext::new(state.clone(), Uuid::new_v4(), "synergy");

        let intervals: Vec<EnergyInterval> = state
            .lua
            .call_integration(&cx, parser, &[serde_json::Value::String(csv)])
            .await?;

        for interval in &intervals {
            let time = interval.time().ok_or_else(|| {
                format!("{parser} returned an out of range epoch {}", interval.at)
            })?;

            state
                .repos
                .energy()
                .record(interval.used, interval.exported, time)
                .await?;
        }

        tracing::info!("recorded {} intervals from {parser}", intervals.len());

        Ok(intervals.len())
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
                    Ok(_) => {
                        crate::metrics::record_integration_poll(
                            "synergy",
                            "success",
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
