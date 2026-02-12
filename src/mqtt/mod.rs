use crate::actors::event_handler;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use rumqttc::MqttOptions;
use serde::Serialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub const ZIGBEE2MQTT_BASE: &str = "zigbee2mqtt";

pub struct Mqtt {
    client: rumqttc::AsyncClient,
    connection: rumqttc::EventLoop,
}

#[derive(thiserror::Error, Debug)]
pub enum MqttError {
    #[error("a mqtt connection error occurred: {0}")]
    MqttConnection(#[from] rumqttc::ConnectionError),

    #[error("a mqtt client error occurred: {0}")]
    Mqtt(#[from] rumqttc::ClientError),

    #[error("a actor message error occurred: {0}")]
    ActorMessage(#[from] ractor::MessagingErr<FactoryMessage<(), event_handler::Message>>),
}

#[derive(Clone)]
pub struct MqttClient {
    client: rumqttc::AsyncClient,
}

impl MqttClient {
    pub fn json_bytes<T>(structure: T) -> Vec<u8>
    where
        T: Serialize,
    {
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, &structure).unwrap();
        bytes
    }

    pub async fn send_event_raw(&self, topic: String, payload: &str) -> Result<(), MqttError> {
        self.client
            .publish(topic, rumqttc::QoS::ExactlyOnce, false, payload)
            .await
            .map_err(MqttError::from)
    }

    pub async fn send_event<T>(&self, topic: String, payload: T) -> Result<(), MqttError>
    where
        T: Serialize,
    {
        self.client
            .publish(
                topic,
                rumqttc::QoS::ExactlyOnce,
                false,
                MqttClient::json_bytes(payload),
            )
            .await
            .map_err(MqttError::from)
    }
}

impl Mqtt {
    pub async fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
    ) -> Result<(MqttClient, Self), MqttError> {
        let client_id = if !cfg!(debug_assertions) {
            "home-gateway-dev"
        } else {
            "home-gateway"
        };

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        // for devices packet
        mqttoptions.set_max_packet_size(100_000, 100_000);
        mqttoptions.set_credentials(username, password);

        let (client, connection) = rumqttc::AsyncClient::new(mqttoptions, 10);

        Ok((
            MqttClient {
                client: client.clone(),
            },
            Self { client, connection },
        ))
    }

    pub async fn process_events(
        &mut self,
        cancellation_token: CancellationToken,
        actor: ActorRef<FactoryMessage<(), event_handler::Message>>,
    ) -> Result<(), MqttError> {
        loop {
            tokio::select! {
                event = self.connection.poll() => {
                    match event {
                        Ok(event) => match event {
                            rumqttc::Event::Incoming(packet) if matches!(packet, rumqttc::Packet::ConnAck(_)) => {
                                self.client
                                    .subscribe("zigbee2mqtt/+", rumqttc::QoS::ExactlyOnce)
                                    .await?;

                                self.client
                                    .subscribe("zigbee2mqtt/bridge/devices", rumqttc::QoS::ExactlyOnce)
                                    .await?;
                            },
                            rumqttc::Event::Incoming(packet) => if let rumqttc::Packet::Publish(publish) = packet {
                                let response = actor.send_message(FactoryMessage::Dispatch(Job {
                                    key: (),
                                    msg: event_handler::Message::MqttPacket {
                                        payload: publish.payload,
                                        topic: publish.topic
                                    },
                                    options: JobOptions::default(),
                                    accepted: None

                                }));

                                if let Err(e) = response {
                                    tracing::error!("error sending to event handler actor: {e}")
                                };
                            }
                            rumqttc::Event::Outgoing(_) => {}
                        }
                        Err(e) => tracing::error!("error with event: {e}"),
                    };
                }
                _ = cancellation_token.cancelled() => {
                    tracing::info!("cancellation requested");
                    break Ok(());
                }
            }
        }
    }
}
