use crate::event_bus::EventBusMessage;
use crate::{db::UnifiState, state::AppState};
use ractor::Actor;
use tracing::instrument;
use types::{Parameters, UnifiWebhookEvent};

pub mod lua;
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
        alias: Option<&str>,
        hostname: Option<&str>,
        state: UnifiState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let previous = self
            .shared_actor_state
            .repos
            .unifi()
            .upsert_state(mac_address, alias, hostname, state)
            .await?;

        if previous == Some(state) {
            tracing::debug!("unifi client {mac_address} already {state:?}, skipping publish");
            return Ok(());
        }

        let client = alias.or(hostname).unwrap_or(mac_address);

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Unifi {
                event_id: uuid::Uuid::new_v4(),
                mac_address: mac_address.to_string(),
                client: client.to_string(),
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
                let (mac_address, alias, hostname) = match event.parameters {
                    Parameters::Connect(p) => (
                        p.unificlient_mac,
                        p.unificlient_alias,
                        p.unificlient_hostname,
                    ),
                    Parameters::Disconnect(p) => (
                        p.unificlient_mac,
                        p.unificlient_alias,
                        p.unificlient_hostname,
                    ),
                };

                let state = match event.name.as_str() {
                    "WiFi Client Connected" | "Wired Client Connected" => UnifiState::Connected,
                    "WiFi Client Disconnected" | "Wired Client Disconnected" => {
                        UnifiState::Disconnected
                    }
                    unknown => {
                        tracing::warn!("unknown webhook event: {unknown}");
                        return Ok(());
                    }
                };

                self.set_client_state(&mac_address, alias.as_deref(), hostname.as_deref(), state)
                    .await?;
            }
        }

        Ok(())
    }
}
