use crate::device_registry::Transport;
use crate::integrations::mqtt::{MqttClient, MqttProtocol};
use crate::{
    decoding::DecodedDevice, integrations::zigbee2mqtt::devices::BridgeDevices, lua::LuaDecoder,
    state::AppState,
};
use mqtt_ingest_state::MqttIngestState;
use mqtt_topic::{MqttTopic, TopicMatch};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde::Serialize;
use serde_json::{Value, json};
use tracing::Instrument;
use uuid::Uuid;

mod mqtt_ingest_state;
mod mqtt_topic;
pub mod spawn;

pub enum Message {
    MqttPacket {
        payload: bytes::Bytes,
        topic: String,
    },
}

struct Report {
    device: DecodedDevice,
    friendly_name: String,
    input: Value,
}

pub struct MqttIngest {
    shared_actor_state: AppState,
}

impl MqttIngest {
    pub const NAME: &str = "mqtt-ingest";

    async fn decode_and_dispatch<I: Serialize>(
        &self,
        decoder: &LuaDecoder,
        device: &DecodedDevice,
        friendly_name: &str,
        input: &I,
    ) {
        let reading = match device.profile.decode(decoder, input) {
            Ok(reading) => reading,
            Err(e) => {
                tracing::error!(
                    "failed to decode {} message for {} with model {}: {e}",
                    device.profile.source(),
                    device.address,
                    device.profile.slug
                );
                crate::tracing_context::record_current_error(&e.to_string());

                return;
            }
        };

        crate::decoding::dispatch(
            &self.shared_actor_state,
            Uuid::new_v4(),
            device,
            friendly_name,
            reading,
        )
        .await;
    }

    async fn record_last_seen(&self, address: &str) {
        crate::device_registry::last_seen::record(
            &self.shared_actor_state.devices,
            self.shared_actor_state.repos.device(),
            address,
        )
        .await;
    }

    async fn handle_directory(&self, payload: &[u8]) -> Result<(), anyhow::Error> {
        let devices_payload = serde_json::from_slice::<BridgeDevices>(payload)?;

        for device in devices_payload {
            let ieee_address = device.ieee_address;
            let friendly_name = device.friendly_name;

            self.shared_actor_state
                .repos
                .device()
                .upsert_known(&ieee_address, &friendly_name)
                .await?;
            self.shared_actor_state
                .devices
                .record_friendly_name(ieee_address, friendly_name)
                .await;
        }

        Ok(())
    }

    async fn handle_discovery(
        &self,
        protocol: MqttProtocol,
        address: String,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let discovery = serde_json::from_slice::<Value>(payload)?;
        let devices = &self.shared_actor_state.devices;

        match discovery.get("friendly_name").and_then(Value::as_str) {
            Some(friendly_name) => {
                tracing::info!("discovered {protocol} device: {friendly_name} ({address})");

                devices
                    .record_friendly_name(address.clone(), friendly_name.to_owned())
                    .await;
            }
            None => {
                tracing::info!("discovered {protocol} device {address} without a friendly name")
            }
        }

        let topics = devices.mqtt_topics_for(&address).to_vec();

        if topics.is_empty() {
            tracing::warn!("discovered {protocol} device {address} is not registered");
        }

        for topic in topics {
            tracing::info!("subscribing to {protocol} topic: {topic}");

            self.shared_actor_state
                .handles
                .expect::<MqttClient>()
                .subscribe(topic)
                .await?;
        }

        Ok(())
    }

    async fn resolve(&self, candidate: TopicMatch, payload: &[u8]) -> Result<Report, String> {
        let TopicMatch {
            protocol,
            stream,
            vars,
        } = candidate;

        let devices = &self.shared_actor_state.devices;

        let Some(value) = protocol.parse_payload(&vars, payload) else {
            return Err(format!("unrecognised {protocol} {stream} payload"));
        };

        let name = vars.get("name").cloned();

        let address = match (vars.get("address"), &name) {
            (Some(address), _) => address.clone(),
            (None, Some(name)) => match protocol.payload_address(&value) {
                Some(address) => address.to_owned(),
                None => devices
                    .address_for_friendly_name(name)
                    .await
                    .unwrap_or_else(|| name.clone()),
            },
            (None, None) => return Err(format!("{protocol} {stream} topic names no device")),
        };

        let Some(device) = devices.mqtt_device(protocol, &address) else {
            return Err(format!(
                "unregistered {protocol} device {address}; add it to the devices config"
            ));
        };

        let friendly_name = match name {
            Some(name) => name,
            None => devices
                .friendly_name(&address)
                .await
                .unwrap_or_else(|| device.id.clone()),
        };

        let input = json!({
            "topic": stream,
            "vars": vars,
            "payload": value,
        });

        Ok(Report {
            device,
            friendly_name,
            input,
        })
    }

    async fn handle_report(
        &self,
        decoder: &LuaDecoder,
        candidates: Vec<TopicMatch>,
        payload: &[u8],
    ) {
        let mut failures = Vec::new();

        for candidate in candidates {
            match self.resolve(candidate, payload).await {
                Ok(Report {
                    device,
                    friendly_name,
                    input,
                }) => {
                    self.record_last_seen(&device.address).await;

                    tracing::info!(
                        "received {} message for {friendly_name} ({}, model {})",
                        device.profile.source(),
                        device.address,
                        device.profile.slug
                    );

                    self.decode_and_dispatch(decoder, &device, &friendly_name, &input)
                        .await;

                    return;
                }
                Err(reason) => failures.push(reason),
            }
        }

        tracing::warn!("ignoring mqtt report: {}", failures.join("; "));
    }

    async fn handle(&self, decoder: &LuaDecoder, message: Message) -> Result<(), anyhow::Error> {
        let Message::MqttPacket { payload, topic } = message;
        let mqtt_topic =
            MqttTopic::classify(self.shared_actor_state.devices.mqtt_protocols(), &topic);

        crate::metrics::record_mqtt_ingest(mqtt_topic.kind());

        match mqtt_topic {
            MqttTopic::Directory(protocol) => {
                tracing::info!("received {protocol} device directory");
                self.handle_directory(&payload).await?;
            }
            MqttTopic::Discovery { protocol, address } => {
                self.handle_discovery(protocol, address, &payload).await?;
            }
            MqttTopic::Report(candidates) => {
                self.handle_report(decoder, candidates, &payload).await;
            }
            MqttTopic::Unhandled => {
                tracing::warn!("ignoring mqtt packet on unhandled topic: {topic}");
            }
        }

        Ok(())
    }
}

impl Worker for MqttIngest {
    type Key = ();
    type Message = Message;
    type State = MqttIngestState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let settings = &self.shared_actor_state.settings;

        let decoder = LuaDecoder::load(
            &Transport::Mqtt.to_string(),
            &settings.model_sources.mqtt,
            &settings.lua,
        )?;

        Ok(MqttIngestState { decoder })
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let Message::MqttPacket { topic, .. } = &msg;
        let topic = topic.clone();
        let topic_kind =
            MqttTopic::classify(self.shared_actor_state.devices.mqtt_protocols(), &topic).kind();

        let span = tracing::info_span!(
            parent: None,
            crate::tracing_setup::MQTT_INGEST_SPAN,
            topic = %topic,
            topic_kind,
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );

        if let Err(e) = Self::handle(self, &state.decoder, msg)
            .instrument(span.clone())
            .await
        {
            tracing::error!("error while handling message: {e}");
            crate::tracing_context::record_error(&span, &e.to_string());

            let _errored = tracing::error_span!(
                parent: None,
                "mqtt.ingest.error",
                force_sample = "true",
                topic = %topic,
                topic_kind,
                otel.status_code = "ERROR",
                otel.status_message = %e,
            )
            .entered();
        }

        Ok(())
    }
}

pub struct MqttMessageHandlerBuilder {
    pub shared_actor_state: AppState,
}
impl WorkerBuilder<MqttIngest, ()> for MqttMessageHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (MqttIngest, ()) {
        (
            MqttIngest {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
