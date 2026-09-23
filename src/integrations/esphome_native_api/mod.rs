mod command;
mod entity_domain;
mod error;
mod frame;
mod node;
mod proto;
mod session;
mod state_update;

use std::collections::HashMap;

use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::actors::system::esphome_native_api_ingest::{self, EsphomeNativeApiIngest};
use crate::media_control::MediaCommand;
use crate::settings::EsphomeSettings;

use command::Command;
use session::Session;

pub use error::EsphomeNativeApiError;
pub use node::Node;
pub use state_update::StateUpdate;

const COMMAND_QUEUE: usize = 16;

#[derive(Clone)]
pub struct EsphomeNativeApi {
    nodes: HashMap<String, mpsc::Sender<Command>>,
}

impl EsphomeNativeApi {
    pub fn new(addresses: impl IntoIterator<Item = String>) -> (Self, Vec<Node>) {
        let mut nodes = HashMap::new();
        let mut channels = Vec::new();

        for address in addresses {
            let (sender, receiver) = mpsc::channel(COMMAND_QUEUE);

            nodes.insert(address.clone(), sender);
            channels.push(Node { address, receiver });
        }

        (Self { nodes }, channels)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub async fn light(&self, address: &str, payload: Value) -> Result<(), EsphomeNativeApiError> {
        self.send(address, Command::Light(payload)).await
    }

    pub async fn media_player(
        &self,
        address: &str,
        command: MediaCommand,
    ) -> Result<(), EsphomeNativeApiError> {
        self.send(address, Command::Media(command)).await
    }

    async fn send(&self, address: &str, command: Command) -> Result<(), EsphomeNativeApiError> {
        let Some(node) = self.nodes.get(address) else {
            return Err(EsphomeNativeApiError::NotConnected {
                address: address.to_owned(),
            });
        };

        node.send(command)
            .await
            .map_err(|_| EsphomeNativeApiError::NotConnected {
                address: address.to_owned(),
            })
    }
}

fn ingest_actor() -> Option<ActorRef<FactoryMessage<String, esphome_native_api_ingest::Message>>> {
    ractor::registry::where_is(EsphomeNativeApiIngest::NAME).map(ActorRef::from)
}

fn dispatch(update: StateUpdate) {
    let address = update.address.clone();

    send(address, esphome_native_api_ingest::Message::State(update));
}

fn is_transition(reported: &mut Option<bool>, connected: bool) -> bool {
    reported.replace(connected) != Some(connected)
}

fn dispatch_connection(reported: &mut Option<bool>, address: &str, connected: bool) {
    if !is_transition(reported, connected) {
        return;
    }

    send(
        address.to_owned(),
        esphome_native_api_ingest::Message::Connection {
            address: address.to_owned(),
            connected,
        },
    );
}

fn send(address: String, msg: esphome_native_api_ingest::Message) {
    let Some(actor) = ingest_actor() else {
        tracing::error!("esphome native api ingest actor is not registered, dropping {address}");
        return;
    };

    let response = actor.send_message(FactoryMessage::Dispatch(Job {
        key: address,
        msg,
        options: JobOptions::default(),
        accepted: None,
    }));

    if let Err(e) = response {
        tracing::error!("error sending to esphome native api ingest actor: {e}");
    }
}

pub async fn process_events(
    mut node: Node,
    settings: EsphomeSettings,
    key: String,
    cancellation_token: CancellationToken,
) {
    let mut reported = None;

    loop {
        tokio::select! {
            result = connected(&mut node, &settings, &key, &mut reported) => {
                if let Err(e) = result {
                    tracing::error!(
                        "esphome node {} error, reconnecting in {:?}: {e}",
                        node.address,
                        settings.reconnect_delay()
                    );
                }

                dispatch_connection(&mut reported, &node.address, false);

                tokio::time::sleep(settings.reconnect_delay()).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("esphome node {} cancellation requested", node.address);
                return;
            }
        }
    }
}

async fn connected(
    node: &mut Node,
    settings: &EsphomeSettings,
    key: &str,
    reported: &mut Option<bool>,
) -> Result<(), EsphomeNativeApiError> {
    let mut session = Session::connect(&node.address, settings, key).await?;

    dispatch_connection(reported, &node.address, true);

    let result = session.run(&mut node.receiver, settings, dispatch).await;

    session.close().await;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_change_in_reachability_is_reported() {
        let mut reported = None;

        assert!(is_transition(&mut reported, false));
        assert!(!is_transition(&mut reported, false));
        assert!(is_transition(&mut reported, true));
        assert!(!is_transition(&mut reported, true));
        assert!(is_transition(&mut reported, false));
    }

    #[test]
    fn a_node_that_never_connects_is_reported_once() {
        let mut reported = None;

        assert!(
            is_transition(&mut reported, false),
            "a node that is down at startup should be reported"
        );
        assert!(
            !is_transition(&mut reported, false),
            "every reconnect attempt should not re-report it"
        );
    }
}
