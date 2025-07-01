use crate::actors::event_handler;
use http::{HeaderMap, Method};
use itertools::Itertools;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use types::{UnifiClient, UnifiConnectedClients, UnifiConnectedClientsResponse};

pub mod types;

pub struct Unifi {
    client: reqwest::Client,
    unifi_site_id: String,
    unifi_api_base: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UnifiError {
    #[error("a actor message error occurred: {0}")]
    ActorMessage(#[from] ractor::MessagingErr<FactoryMessage<(), event_handler::Message>>),
    #[error("a reqwest error occurred: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

impl Unifi {
    const GET_CONNECTED_CLIENTS: &str =
        "{apiBase}/proxy/network/integration/v1/sites/{siteId}/clients";
    pub fn new(
        unifi_api_key: String,
        unifi_site_id: String,
        unifi_api_base: String,
    ) -> Result<Self, UnifiError> {
        let mut headers = HeaderMap::with_capacity(1);
        headers.insert("X-API-KEY", unifi_api_key.parse().unwrap());

        Ok(Self {
            client: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .default_headers(headers)
                .timeout(Duration::from_secs(30))
                .build()?,
            unifi_site_id,
            unifi_api_base,
        })
    }

    async fn get_connected_clients(&self) -> Result<UnifiConnectedClientsResponse, UnifiError> {
        let url = Self::GET_CONNECTED_CLIENTS
            .replace("{apiBase}", &self.unifi_api_base)
            .replace("{siteId}", &self.unifi_site_id);

        let response = self
            .client
            .request(Method::GET, url)
            .send()
            .await?
            .error_for_status()?
            .json::<UnifiConnectedClientsResponse>()
            .await?;

        Ok(response)
    }

    pub async fn process_events(
        &self,
        cancellation_token: CancellationToken,
        actor: ActorRef<FactoryMessage<(), event_handler::Message>>,
    ) -> Result<(), UnifiError> {
        loop {
            let actor = actor.clone();
            let fut = async move {
                let response = self.get_connected_clients().await;
                if let Err(e) = response {
                    tracing::error!("error fetching from unifi: {e}");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    return Ok(());
                }

                let response = response.unwrap();
                if let Err(e) = actor.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: event_handler::Message::UnifiClients {
                        payload: UnifiConnectedClients {
                            clients: response
                                .data
                                .into_iter()
                                .map(|cc| UnifiClient {
                                    id: cc.id,
                                    name: cc.name,
                                    connected_at: cc.connected_at,
                                    ip_address: cc.ip_address,
                                    type_field: cc.type_field,
                                })
                                .collect_vec(),
                        },
                    },
                    options: JobOptions::default(),
                    accepted: None,
                })) {
                    tracing::error!("error sending to event handler actor: {e}")
                };

                tokio::time::sleep(Duration::from_secs(60)).await;
                Ok::<(), UnifiError>(())
            };

            tokio::select! {
                result = fut => {
                    if let Err(e) = result {
                        tracing::error!("error fetching from unifi: {e}")
                    }

                }
                _ = cancellation_token.cancelled() => {
                    tracing::info!("cancellation requested");
                    break Ok(());
                }
            }
        }
    }
}
