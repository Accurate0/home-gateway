use crate::actors::devices::{control_switch, plant_sensor, presence_sensor, robot_vacuum};
use crate::{
    actors::devices::{
        door_sensor, environment_sensor, environment_sensor::EnvironmentSensorHandler, light,
        presence_sensor::PresenceSensorHandler, smart_switch,
    },
    device_metric::DeviceMetric,
    device_registry::ZigbeeDevice,
    state::SharedActorState,
    integrations::zigbee2mqtt::{devices::BridgeDevices, role},
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use serde_json::{Map, Value};
use uuid::Uuid;

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

        if let Some(rest) = topic.strip_prefix("valetudo/") {
            if let Some((identifier, leaf)) = rest.split_once('/') {
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
        }

        MqttTopic::Other
    }
}

pub struct MqttIngest {
    shared_actor_state: SharedActorState,
}

impl MqttIngest {
    pub const NAME: &str = "mqtt-ingest";

    /// Route an esphome motion (`binary_sensor`) reading to the presence actor.
    async fn dispatch_esphome_light(
        &self,
        node: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let Some(on) = crate::integrations::esphome::parse_light_state(payload) else {
            tracing::warn!("unrecognised esphome light state payload for {node}");
            return Ok(());
        };

        crate::actors::devices::light::record_light_state(
            &self.shared_actor_state,
            uuid::Uuid::new_v4(),
            node.to_string(),
            if on { "ON" } else { "OFF" }.to_string(),
        )
        .await
    }

    fn dispatch_esphome_motion(
        &self,
        node: &str,
        object_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let Some(motion) = crate::integrations::esphome::parse_binary_state(payload) else {
            tracing::warn!("unrecognised esphome binary state payload for {node}");
            return Ok(());
        };

        let event_id = uuid::Uuid::new_v4();
        let Some(actor_cell) = ractor::registry::where_is(PresenceSensorHandler::NAME) else {
            tracing::error!("no presence sensor actor found for esphome motion");
            return Ok(());
        };

        actor_cell.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: presence_sensor::Message::NewEvent(presence_sensor::NewEvent {
                event_id,
                entity: presence_sensor::Entity::Esphome {
                    node: node.to_string(),
                    object_id: object_id.to_string(),
                    motion,
                },
            }),
            options: JobOptions::default(),
            accepted: None,
        }))?;

        Ok(())
    }

    /// Route an esphome scalar `sensor` reading to whichever of the plant /
    /// environment actors claim the node — a node can be configured as both, and
    /// each actor ignores object_ids it doesn't handle.
    fn dispatch_esphome_sensor(
        &self,
        node: &str,
        object_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let Some(value) = crate::integrations::esphome::parse_sensor_state(payload) else {
            tracing::warn!("unrecognised esphome sensor payload for {node}/{object_id}");
            return Ok(());
        };

        let event_id = uuid::Uuid::new_v4();

        if self.shared_actor_state.devices.plant(node).is_some() {
            match ractor::registry::where_is(plant_sensor::PlantSensorHandler::NAME.to_string()) {
                Some(actor_cell) => {
                    actor_cell.send_message(FactoryMessage::Dispatch(Job {
                        key: (),
                        msg: plant_sensor::Message::NewEvent(plant_sensor::NewEvent {
                            event_id,
                            node: node.to_string(),
                            object_id: object_id.to_string(),
                            value,
                        }),
                        options: JobOptions::default(),
                        accepted: None,
                    }))?;
                }
                None => tracing::error!("no plant sensor actor found for esphome sensor"),
            }
        }

        if self.shared_actor_state.devices.environment(node).is_some() {
            match ractor::registry::where_is(EnvironmentSensorHandler::NAME) {
                Some(actor_cell) => {
                    actor_cell.send_message(FactoryMessage::Dispatch(Job {
                        key: (),
                        msg: environment_sensor::Message::NewEvent(Box::new(
                            environment_sensor::NewEvent {
                                event_id,
                                entity: environment_sensor::Entity::Esphome {
                                    node: node.to_string(),
                                    object_id: object_id.to_string(),
                                    value,
                                },
                            },
                        )),
                        options: JobOptions::default(),
                        accepted: None,
                    }))?;
                }
                None => tracing::error!("no temperature sensor actor found for esphome sensor"),
            }
        }

        Ok(())
    }

    fn record_battery(&self, address: &str, percent: f64) {
        let devices = &self.shared_actor_state.devices;

        let Some(settings) = devices.battery(address) else {
            return;
        };

        let device_id = devices
            .id_for_address(address)
            .unwrap_or(address)
            .to_owned();

        crate::actors::system::battery::BatteryActor::report(
            device_id,
            settings.name.clone(),
            "battery".to_owned(),
            None,
            Some(percent),
            None,
        );
    }

    async fn dispatch_zigbee(
        &self,
        event_id: Uuid,
        device: &ZigbeeDevice,
        friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Result<(), anyhow::Error> {
        let address = device.address.clone();
        let devices = &self.shared_actor_state.devices;

        if devices.battery(&address).is_some()
            && let Some(percent) = role::battery(device, payload)
        {
            self.record_battery(&address, percent as f64);
        }

        role::run::<door_sensor::Entity>(event_id, devices, device, friendly_name, payload);
        role::run::<environment_sensor::Entity>(event_id, devices, device, friendly_name, payload);
        role::run::<light::Entity>(event_id, devices, device, friendly_name, payload);
        role::run::<smart_switch::Entity>(event_id, devices, device, friendly_name, payload);
        role::run::<presence_sensor::Entity>(event_id, devices, device, friendly_name, payload);
        role::run::<control_switch::Entity>(event_id, devices, device, friendly_name, payload);

        let device_id = devices.id_for_address(&address).map(str::to_owned);

        for (metric, value) in role::metrics(device, payload) {
            let record = DeviceMetric {
                event_id,
                address: address.clone(),
                device_id: device_id.clone(),
                metric,
                value,
            };

            if let Err(e) = record.save(&self.shared_actor_state.db).await {
                tracing::error!("failed to save device metric for {address}: {e}");
            }
        }

        Ok(())
    }

    async fn record_last_seen(&self, device_key: &str) {
        if let Err(e) = sqlx::query!(
            "INSERT INTO device_last_seen (device_key, last_seen) VALUES ($1, now()) \
             ON CONFLICT (device_key) DO UPDATE SET last_seen = now()",
            device_key
        )
        .execute(&self.shared_actor_state.db)
        .await
        {
            tracing::error!("failed to record last seen for {device_key}: {e}");
        }
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        let Message::MqttPacket { payload, topic } = message;
        match MqttTopic::classify(&topic) {
            MqttTopic::Zigbee2MqttBridgeDevices => {
                let devices_payload = serde_json::from_slice::<BridgeDevices>(&payload)?;
                for device in devices_payload {
                    let ieee_address = device.ieee_address;
                    let friendly_name = device.friendly_name;
                    sqlx::query!(
                            "INSERT INTO known_devices (ieee_addr, name) VALUES ($1, $2) ON CONFLICT (ieee_addr) DO UPDATE SET name = $2",
                                &ieee_address,
                                &friendly_name
                            )
                            .execute(&self.shared_actor_state.db)
                            .await?;
                    self.shared_actor_state
                        .devices
                        .record_friendly_name(ieee_address, friendly_name)
                        .await;
                }
            }
            MqttTopic::EsphomeDiscovery => {
                let discovery =
                    serde_json::from_slice::<crate::integrations::esphome::EsphomeDiscovery>(&payload)?;
                tracing::info!(
                    "discovered esphome device: {} ({})",
                    discovery.friendly_name,
                    discovery.name
                );

                self.shared_actor_state
                    .devices
                    .record_friendly_name(discovery.name.clone(), discovery.friendly_name)
                    .await;

                let topics: Vec<String> = self
                    .shared_actor_state
                    .devices
                    .esphome_topics_for(&discovery.name)
                    .map(|(topic, _)| topic.clone())
                    .collect();

                for topic in topics {
                    tracing::info!("subscribing to esphome topic: {topic}");
                    self.shared_actor_state.mqtt.subscribe(topic).await?;
                }
            }
            MqttTopic::Other => {
                // the only non-control topics we subscribe to are esphome state
                // topics declared in the sensor registry; look the target up exactly
                let target = self
                    .shared_actor_state
                    .devices
                    .esphome_target(&topic)
                    .cloned();

                match target {
                    Some(crate::integrations::esphome::EsphomeTarget::Motion { node, object_id }) => {
                        self.record_last_seen(&node).await;
                        self.dispatch_esphome_motion(&node, &object_id, &payload)?
                    }
                    Some(crate::integrations::esphome::EsphomeTarget::Sensor { node, object_id }) => {
                        self.record_last_seen(&node).await;
                        self.dispatch_esphome_sensor(&node, &object_id, &payload)?
                    }
                    Some(crate::integrations::esphome::EsphomeTarget::Light { node, .. }) => {
                        self.record_last_seen(&node).await;
                        self.dispatch_esphome_light(&node, &payload).await?
                    }
                    None => {
                        tracing::warn!("ignoring mqtt packet on unhandled topic: {topic}")
                    }
                }
            }
            MqttTopic::Valetudo { identifier, leaf } => {
                self.record_last_seen(&identifier).await;

                let Some(actor_cell) =
                    ractor::registry::where_is(robot_vacuum::RobotVacuumHandler::NAME.to_string())
                else {
                    tracing::error!("no robot vacuum actor found");
                    return Ok(());
                };

                let device_id = self
                    .shared_actor_state
                    .devices
                    .id_for_address(&identifier)
                    .unwrap_or(&identifier)
                    .to_owned();

                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: robot_vacuum::Message::Valetudo(robot_vacuum::ValetudoEvent {
                        event_id: uuid::Uuid::new_v4(),
                        device_id,
                        leaf,
                        payload,
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            MqttTopic::Zigbee2MqttDevice => {
                let friendly_name = topic
                    .strip_prefix("zigbee2mqtt/")
                    .unwrap_or(&topic)
                    .to_owned();

                self.record_last_seen(&friendly_name).await;

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

                let Some(device) = devices.zigbee_device(&address).cloned() else {
                    tracing::warn!(
                        "unregistered zigbee device {address} ({friendly_name}); add it to devices.yaml"
                    );
                    return Ok(());
                };

                tracing::info!(
                    "received zigbee message for {friendly_name} ({}, model {})",
                    address,
                    device.profile.slug
                );

                self.dispatch_zigbee(Uuid::new_v4(), &device, &friendly_name, object)
                    .await?;
            }
        }

        Ok(())
    }
}

impl Worker for MqttIngest {
    type Key = ();
    type Message = Message;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct MqttMessageHandlerBuilder {
    pub shared_actor_state: SharedActorState,
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
        // esphome state topics are not classified by shape any more — they fall
        // through to `Other` and are resolved against the subscription registry
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
