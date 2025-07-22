use crate::types::{SharedActorState, db::UnifiState};
use ractor::Actor;
use tracing::instrument;

pub mod types;

pub enum UnifiMessage {
    ClientDisconnect { mac_address: String },
    ClientConnect { mac_address: String },
}

pub struct UnifiConnectedClientHandler {
    pub shared_actor_state: SharedActorState,
}

impl UnifiConnectedClientHandler {
    pub const NAME: &str = "unifi-connected-clients";
}

impl UnifiConnectedClientHandler {
    #[instrument(skip(self))]
    async fn set_client_state(
        &self,
        mac_address: &str,
        state: UnifiState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let event_id = uuid::Uuid::new_v4();
        let response = sqlx::query!(
            "SELECT name, id FROM unifi_clients_mapping WHERE mac_address = $1",
            mac_address
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        let name = response
            .as_ref()
            .map(|r| r.name.as_str())
            .unwrap_or("unknown");

        let id = response
            .as_ref()
            .map(|r| r.id.to_string())
            .unwrap_or("unknown".to_owned());

        sqlx::query!(
            "INSERT INTO unifi_clients (event_id, name, id, state) VALUES ($1, $2, $3, $4)",
            event_id,
            name,
            id,
            state as UnifiState
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }
}

impl Actor for UnifiConnectedClientHandler {
    type Msg = UnifiMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "unifi-connected-clients", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            UnifiMessage::ClientDisconnect { mac_address } => {
                self.set_client_state(&mac_address, UnifiState::Disconnected)
                    .await?;
            }
            UnifiMessage::ClientConnect { mac_address } => {
                self.set_client_state(&mac_address, UnifiState::Connected)
                    .await?;
            }
        }

        Ok(())
    }
}
