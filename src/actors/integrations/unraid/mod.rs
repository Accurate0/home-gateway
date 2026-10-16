use ractor::Actor;
use uuid::Uuid;

use crate::event_bus::EventBusMessage;
use crate::integrations::unraid::Unraid;
use crate::integrations::unraid::types::{UnraidEvent, UnraidSnapshot};
use crate::metrics::record_integration_poll;
use crate::settings::UnraidSettings;
use crate::state::AppState;
use crate::telemetry::context::record_current_error;

pub enum UnraidMessage {
    Poll,
}

pub struct UnraidActorState {
    previous: Option<UnraidSnapshot>,
}

pub struct UnraidActor {
    pub shared_actor_state: AppState,
    pub unraid: Unraid,
}

impl UnraidActor {
    pub const NAME: &str = "unraid";

    async fn poll(&self, state: &mut UnraidActorState) -> Result<(), ractor::ActorProcessingErr> {
        let snapshot = self.unraid.snapshot().await?;

        self.shared_actor_state.repos.unraid().record(&snapshot).await?;

        let events = UnraidEvent::between(state.previous.as_ref(), &snapshot);

        tracing::info!(
            "recorded unraid array {} with {} disks, publishing {} events",
            snapshot.array.state,
            snapshot.disks.len(),
            events.len()
        );

        for event in events {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Unraid {
                    event_id: Uuid::new_v4(),
                    subject: event.subject,
                    kind: event.kind,
                    update: event.update,
                });
        }

        state.previous = Some(snapshot);

        Ok(())
    }
}

impl Actor for UnraidActor {
    type Msg = UnraidMessage;
    type State = UnraidActorState;
    type Arguments = UnraidSettings;

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        settings: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let previous = self
            .shared_actor_state
            .repos
            .unraid()
            .latest()
            .await?
            .map(|stored| stored.snapshot);

        myself.cast(UnraidMessage::Poll)?;
        myself.send_interval(settings.refresh.to_std()?, || UnraidMessage::Poll);

        Ok(UnraidActorState { previous })
    }

    #[tracing::instrument(
        parent = None,
        name = "actor.unraid",
        skip(self, _myself, message, state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            UnraidMessage::Poll => {
                let started = std::time::Instant::now();

                match self.poll(state).await {
                    Ok(()) => {
                        record_integration_poll("unraid", "success", started.elapsed());
                    }
                    Err(e) => {
                        tracing::error!("error polling unraid: {e}");
                        record_current_error(&e.to_string());
                        record_integration_poll("unraid", "error", started.elapsed());
                    }
                }
            }
        }

        Ok(())
    }
}
