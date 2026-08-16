use crate::actors::system::mqtt_ingest;
use crate::device_registry::DeviceRegistry;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use rumqttc::MqttOptions;
use serde::Serialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

fn mqtt_ingest_actor() -> Option<ActorRef<FactoryMessage<(), mqtt_ingest::Message>>> {
    ractor::registry::where_is(mqtt_ingest::MqttIngest::NAME).map(ActorRef::from)
}

pub const ZIGBEE2MQTT_BASE: &str = "zigbee2mqtt";

const STATIC_TOPICS: [&str; 5] = [
    "zigbee2mqtt/+",
    "zigbee2mqtt/bridge/devices",
    "esphome/discover/+",
    "valetudo/+/state",
    "valetudo/+/attributes",
];

const RECONNECT_BACKOFF_MIN: Duration = Duration::from_secs(1);
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(60);

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
    ActorMessage(#[from] Box<ractor::MessagingErr<FactoryMessage<(), mqtt_ingest::Message>>>),
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

    pub async fn subscribe(&self, topic: String) -> Result<(), MqttError> {
        self.client
            .subscribe(topic, rumqttc::QoS::ExactlyOnce)
            .await
            .map_err(MqttError::from)
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
        let client_id = if cfg!(debug_assertions) {
            "home-gateway-dev"
        } else {
            "home-gateway"
        };

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        // for devices packet
        mqttoptions.set_max_packet_size(100_000, 100_000);
        mqttoptions.set_credentials(username, password);

        let (client, connection) = rumqttc::AsyncClient::new(mqttoptions, 100);

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
        devices: DeviceRegistry,
    ) -> Result<(), MqttError> {
        let mut backoff = RECONNECT_BACKOFF_MIN;

        loop {
            tokio::select! {
                event = self.connection.poll() => {
                    match event {
                        Ok(event) => match event {
                            rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                                backoff = RECONNECT_BACKOFF_MIN;

                                let topics: Vec<(&'static str, String)> = STATIC_TOPICS
                                    .iter()
                                    .map(|topic| ("topic", (*topic).to_owned()))
                                    .chain(
                                        devices
                                            .esphome_all_topics()
                                            .map(|topic| ("esphome state topic", topic.clone())),
                                    )
                                    .collect();

                                let client = self.client.clone();

                                tokio::spawn(async move {
                                    for (label, topic) in topics {
                                        tracing::info!("subscribing to {label}: {topic}");

                                        if let Err(e) = client
                                            .subscribe(&topic, rumqttc::QoS::ExactlyOnce)
                                            .await
                                        {
                                            tracing::error!("failed to subscribe to {topic}: {e}");
                                        }
                                    }
                                });
                            },
                            rumqttc::Event::Incoming(packet) => if let rumqttc::Packet::Publish(publish) = packet {
                                let Some(actor) = mqtt_ingest_actor() else {
                                    tracing::error!("mqtt ingest actor is not registered, dropping packet on {}", publish.topic);
                                    continue;
                                };

                                let response = actor.send_message(FactoryMessage::Dispatch(Job {
                                    key: (),
                                    msg: mqtt_ingest::Message::MqttPacket {
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
                        Err(e) => {
                            tracing::error!("error with event, reconnecting in {backoff:?}: {e}");
                            tokio::time::sleep(backoff).await;
                            backoff = (backoff * 2).min(RECONNECT_BACKOFF_MAX);
                        }
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
