use crate::event_bus::EventBusMessage;
use crate::{db::UnifiState, state::AppState};
use ractor::Actor;
use tracing::instrument;
use types::{Parameters, UnifiWebhookEvent};

pub mod types;

pub enum UnifiMessage {
    /// A raw connect/disconnect webhook from the UniFi controller. Parsing the
    /// client mac and connection state out of it lives here rather than in the
    /// HTTP route, so the route is just a thin authenticated forwarder.
    Webhook(Box<UnifiWebhookEvent>),
}

pub struct UnifiConnectedClientHandler {
    pub shared_actor_state: AppState,
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
        let response = self
            .shared_actor_state
            .repos
            .unifi()
            .mapping_for(mac_address)
            .await?;

        let name = response
            .as_ref()
            .map(|r| r.name.as_str())
            .unwrap_or("unknown");

        let id = response
            .as_ref()
            .map(|r| r.id.to_string())
            .unwrap_or("unknown".to_owned());

        self.shared_actor_state
            .repos
            .unifi()
            .append_event(event_id, name, &id, state)
            .await?;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Unifi {
                event_id,
                mac_address: mac_address.to_string(),
                client: name.to_string(),
                connected: matches!(state, UnifiState::Connected),
            });

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
            UnifiMessage::Webhook(event) => {
                let mac_address = match event.parameters {
                    Parameters::Connect(p) => p.unificlient_mac,
                    Parameters::Disconnect(p) => p.unificlient_mac,
                };

                match event.name.as_str() {
                    "WiFi Client Connected" => {
                        self.set_client_state(&mac_address, UnifiState::Connected)
                            .await?;
                    }
                    "WiFi Client Disconnected" => {
                        self.set_client_state(&mac_address, UnifiState::Disconnected)
                            .await?;
                    }
                    unknown => tracing::warn!("unknown webhook event: {unknown}"),
                }
            }
        }

        Ok(())
    }
}
