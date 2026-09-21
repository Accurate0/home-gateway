use crate::actors::devices::robot_vacuum;
use crate::actors::system::rpc;
use crate::integrations::esphome::EsphomeTarget;
use crate::integrations::mqtt::MqttClient;
use crate::{
    decoding::DecodedDevice, integrations::zigbee2mqtt::devices::BridgeDevices, lua::LuaDecoder,
    state::AppState,
};
use decoders::Decoders;
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde::Serialize;
use serde_json::{Value, json};
use tracing::Instrument;
use uuid::Uuid;

mod decoders;
pub mod spawn;

/// Messages handled by the MQTT router worker. The worker's sole job is to
/// decode an incoming MQTT packet and forward a typed event to the device actor
/// that owns it; HTTP webhook ingests talk to their target actors directly.
pub enum Message {
    MqttPacket {
        payload: bytes::Bytes,
        topic: String,
    },
}

/// What an incoming MQTT topic maps to. zigbee2mqtt and esphome share the broker
/// but never the topic space: zigbee2mqtt always publishes under `zigbee2mqtt/`,
/// while esphome uses `esphome/discover/...` for discovery and
/// `<node>/<platform>/<object_id>/state` for entity state. Classifying the topic
/// up front means the producer is decided by an explicit rule rather than by an
/// "everything else is zigbee" fallthrough.
enum MqttTopic {
    /// `zigbee2mqtt/bridge/devices` — the retained device list.
    Zigbee2MqttBridgeDevices,
    /// `zigbee2mqtt/<friendly_name>` — a device's state report.
    Zigbee2MqttDevice,
    /// `esphome/discover/<node>` — a node announcing itself.
    EsphomeDiscovery,
    /// `valetudo/<identifier>/state` or `.../attributes` — a Valetudo robot's
    /// state report.
    Valetudo {
        identifier: String,
        leaf: robot_vacuum::Leaf,
    },
    /// Anything else — resolved against the esphome subscription registry, since
    /// the only other topics we subscribe to are esphome state topics we chose.
    Other,
}

impl MqttTopic {
    fn classify(topic: &str) -> Self {
        if let Some(rest) = topic.strip_prefix("zigbee2mqtt/") {
            return if rest == "bridge/devices" {
                MqttTopic::Zigbee2MqttBridgeDevices
            } else {
                MqttTopic::Zigbee2MqttDevice
            };
        }

        if topic.starts_with("esphome/discover/") {
            return MqttTopic::EsphomeDiscovery;
        }

        if let Some(rest) = topic.strip_prefix("valetudo/")
            && let Some((identifier, leaf)) = rest.split_once('/')
        {
            let leaf = match leaf {
                "state" => Some(robot_vacuum::Leaf::State),
                "attributes" => Some(robot_vacuum::Leaf::Attributes),
                _ => None,
            };
            if let Some(leaf) = leaf {
                return MqttTopic::Valetudo {
                    identifier: identifier.to_owned(),
                    leaf,
                };
            }
        }

        MqttTopic::Other
    }

    fn kind(&self) -> &'static str {
        match self {
            MqttTopic::Zigbee2MqttBridgeDevices => "zigbee2mqtt_bridge_devices",
            MqttTopic::Zigbee2MqttDevice => "zigbee2mqtt_device",
            MqttTopic::EsphomeDiscovery => "esphome_discovery",
            MqttTopic::Valetudo { .. } => "valetudo",
            MqttTopic::Other => "other",
        }
    }
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
                    device.profile.transport,
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

    async fn handle_esphome_state(
        &self,
        decoder: &LuaDecoder,
        target: EsphomeTarget,
        payload: &[u8],
    ) {
        let devices = &self.shared_actor_state.devices;

        let Some(device) = devices.decoded(&target.node) else {
            tracing::warn!("esphome topic for unregistered node {}", target.node);
            return;
        };

        self.record_last_seen(&target.node).await;

        let Some(state) = target.domain.parse(payload) else {
            tracing::warn!(
                "unrecognised esphome {} payload for {}/{}",
                target.domain,
                target.node,
                target.object_id
            );
            return;
        };

        let friendly_name = devices
            .friendly_name(&target.node)
            .await
            .unwrap_or_else(|| device.id.clone());

        let input = json!({
            "domain": target.domain,
            "object_id": target.object_id,
            "state": state,
        });

        self.decode_and_dispatch(decoder, &device, &friendly_name, &input)
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

    async fn handle(&self, decoders: &Decoders, message: Message) -> Result<(), anyhow::Error> {
        let Message::MqttPacket { payload, topic } = message;
        let mqtt_topic = MqttTopic::classify(&topic);
        crate::metrics::record_mqtt_ingest(mqtt_topic.kind());
        match mqtt_topic {
            MqttTopic::Zigbee2MqttBridgeDevices => {
                let devices_payload = serde_json::from_slice::<BridgeDevices>(&payload)?;
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
            }
            MqttTopic::EsphomeDiscovery => {
                let discovery = serde_json::from_slice::<
                    crate::integrations::esphome::EsphomeDiscovery,
                >(&payload)?;
                tracing::info!(
                    "discovered esphome device: {} ({})",
                    discovery.friendly_name,
                    discovery.name
                );

                self.shared_actor_state
                    .devices
                    .record_friendly_name(discovery.name.clone(), discovery.friendly_name)
                    .await;

                let topics = self
                    .shared_actor_state
                    .devices
                    .esphome_topics_for(&discovery.name)
                    .to_vec();

                for topic in topics {
                    tracing::info!("subscribing to esphome topic: {topic}");
                    self.shared_actor_state
                        .handles
                        .expect::<MqttClient>()
                        .subscribe(topic)
                        .await?;
                }
            }
            MqttTopic::Other => {
                let target = self
                    .shared_actor_state
                    .devices
                    .esphome_target(&topic)
                    .cloned();

                match target {
                    Some(target) => {
                        self.handle_esphome_state(&decoders.esphome, target, &payload)
                            .await
                    }
                    None => {
                        tracing::warn!("ignoring mqtt packet on unhandled topic: {topic}")
                    }
                }
            }
            MqttTopic::Valetudo { identifier, leaf } => {
                self.record_last_seen(&identifier).await;

                let device_id = self
                    .shared_actor_state
                    .devices
                    .id_for_address(&identifier)
                    .unwrap_or(&identifier)
                    .to_owned();

                rpc::cast_factory(
                    robot_vacuum::RobotVacuumHandler::NAME,
                    robot_vacuum::Message::Valetudo(robot_vacuum::ValetudoEvent {
                        event_id: uuid::Uuid::new_v4(),
                        traceparent: crate::tracing_context::inject_current(),
                        device_id,
                        leaf,
                        payload,
                    }),
                )?;
            }
            MqttTopic::Zigbee2MqttDevice => {
                let friendly_name = topic
                    .strip_prefix("zigbee2mqtt/")
                    .unwrap_or(&topic)
                    .to_owned();

                let value = serde_json::from_slice::<Value>(&payload)?;
                let Some(object) = value.as_object() else {
                    tracing::warn!("ignoring non-object zigbee payload on {topic}");
                    return Ok(());
                };

                let devices = &self.shared_actor_state.devices;
                let address = match object
                    .get("device")
                    .and_then(|device| device.get("ieee_addr"))
                    .and_then(Value::as_str)
                {
                    Some(address) => address.to_owned(),
                    None => devices
                        .address_for_friendly_name(&friendly_name)
                        .await
                        .unwrap_or_else(|| friendly_name.clone()),
                };

                let Some(device) = devices.zigbee_device(&address) else {
                    tracing::warn!(
                        "unregistered zigbee device {address} ({friendly_name}); add it to devices.yaml"
                    );
                    return Ok(());
                };

                self.record_last_seen(&address).await;

                tracing::info!(
                    "received zigbee message for {friendly_name} ({}, model {})",
                    address,
                    device.profile.slug
                );

                self.decode_and_dispatch(&decoders.zigbee, &device, &friendly_name, object)
                    .await;
            }
        }

        Ok(())
    }
}

impl Worker for MqttIngest {
    type Key = ();
    type Message = Message;
    type State = Decoders;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let settings = &self.shared_actor_state.settings;

        Ok(Decoders::load(&settings.model_sources, &settings.lua)?)
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        decoders: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let Message::MqttPacket { topic, .. } = &msg;
        let topic = topic.clone();
        let topic_kind = MqttTopic::classify(&topic).kind();

        let span = tracing::info_span!(
            parent: None,
            crate::tracing_setup::MQTT_INGEST_SPAN,
            topic = %topic,
            topic_kind,
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );

        if let Err(e) = Self::handle(self, decoders, msg)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_routes_control_topics_and_defers_the_rest() {
        assert!(matches!(
            MqttTopic::classify("zigbee2mqtt/bridge/devices"),
            MqttTopic::Zigbee2MqttBridgeDevices
        ));
        assert!(matches!(
            MqttTopic::classify("zigbee2mqtt/0x00158d008bbe0316"),
            MqttTopic::Zigbee2MqttDevice
        ));
        assert!(matches!(
            MqttTopic::classify("esphome/discover/apollo-mtr-1-livingroom"),
            MqttTopic::EsphomeDiscovery
        ));
        assert!(matches!(
            MqttTopic::classify("apollo-mtr-1-livingroom/sensor/air_temperature/state"),
            MqttTopic::Other
        ));
        assert!(matches!(
            MqttTopic::classify("apollo-mtr-1-livingroom/binary_sensor/ld2450_moving_target/state"),
            MqttTopic::Other
        ));
        assert!(matches!(
            MqttTopic::classify("valetudo/rockrobo/state"),
            MqttTopic::Valetudo {
                leaf: robot_vacuum::Leaf::State,
                ..
            }
        ));
        assert!(matches!(
            MqttTopic::classify("valetudo/rockrobo/attributes"),
            MqttTopic::Valetudo {
                leaf: robot_vacuum::Leaf::Attributes,
                ..
            }
        ));
        assert!(matches!(
            MqttTopic::classify("valetudo/rockrobo/map_data"),
            MqttTopic::Other
        ));
    }
}
