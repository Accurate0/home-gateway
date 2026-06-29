use super::devices::{control_switch, plant_sensor, presence_sensor};
use crate::{
    actors::{door_sensor, environment_sensor, light, smart_switch},
    types::SharedActorState,
    zigbee2mqtt::devices::BridgeDevices,
};
use ractor::{
    ActorCell, ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use types::{GenericZigbee2MqttMessage, TypedActorName};
use uuid::Uuid;

pub mod spawn;
mod types;

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

        MqttTopic::Other
    }
}

pub struct MqttIngest {
    shared_actor_state: SharedActorState,
}

impl MqttIngest {
    pub const NAME: &str = "mqtt-ingest";

    fn handle_control_switch(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::AqaraSingleButtonSwitch(aqara) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: control_switch::ControlSwitchMessage::NewEvent(control_switch::NewEvent {
                        event_id,
                        entity: control_switch::Entity::AqaraSingleButton(aqara),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::IKEASwitch(ikea_e2001) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: control_switch::ControlSwitchMessage::NewEvent(control_switch::NewEvent {
                        event_id,
                        entity: control_switch::Entity::IKEASwitch(ikea_e2001),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!(
                    "actor name ({actor_type}) does not match message for control switch"
                );
            }
        }

        Ok(())
    }

    fn handle_presence_sensor(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::AqaraPresenceSensor(aqara_presence) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: presence_sensor::Message::NewEvent(presence_sensor::NewEvent {
                        event_id,
                        entity: presence_sensor::Entity::AqaraFP1E(Box::new(aqara_presence)),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!("actor name ({actor_type}) does not match message for smart switch");
            }
        }

        Ok(())
    }

    fn handle_smart_switch(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::TS011FSmartSwitch(ts011f_plug1) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: smart_switch::Message::NewEvent(smart_switch::NewEvent {
                        event_id,
                        entity: smart_switch::Entity::TS011FSmartSwitch(ts011f_plug1),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!("actor name ({actor_type}) does not match message for smart switch");
            }
        }

        Ok(())
    }

    fn handle_environment_sensor(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::IKEATemperatureSensor(ikea_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: environment_sensor::Message::NewEvent(Box::new(
                        environment_sensor::NewEvent {
                            event_id,
                            entity: environment_sensor::Entity::IKEAE2112(ikea_temperature_sensor),
                        },
                    )),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(aqara_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: environment_sensor::Message::NewEvent(Box::new(
                        environment_sensor::NewEvent {
                            event_id,
                            entity: environment_sensor::Entity::AqaraWSDCGQ12LM(
                                aqara_temperature_sensor,
                            ),
                        },
                    )),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }

            GenericZigbee2MqttMessage::LumiTemperatureSensor(lumi_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: environment_sensor::Message::NewEvent(Box::new(
                        environment_sensor::NewEvent {
                            event_id,
                            entity: environment_sensor::Entity::LumiWSDCGQ11LM(
                                lumi_temperature_sensor,
                            ),
                        },
                    )),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!(
                    "actor name ({actor_type}) does not match message for temperature sensor"
                );
            }
        }

        Ok(())
    }

    fn handle_door_sensor(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::AquaraDoorSensor(aqara_mccgq12_lm) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: door_sensor::Message::NewEvent(door_sensor::NewEvent {
                        event_id,
                        entity: door_sensor::Entity::AqaraMCCGQ12LM(aqara_mccgq12_lm),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!("actor name ({actor_type}) does not match message for door sensor");
            }
        }

        Ok(())
    }

    fn handle_light(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::PhillipsLight(phillips_light) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: light::LightHandlerMessage::NewEvent(Box::new(light::NewEvent {
                        event_id,
                        entity: light::Entity::Phillips9290012573A(phillips_light),
                    })),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::IKEALight(ikea_light) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: light::LightHandlerMessage::NewEvent(Box::new(light::NewEvent {
                        event_id,
                        entity: light::Entity::IKEALED2201G8(ikea_light),
                    })),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::AqaraWhiteLight(aqara_light) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: light::LightHandlerMessage::NewEvent(Box::new(light::NewEvent {
                        event_id,
                        entity: light::Entity::AqaraT1(aqara_light),
                    })),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            _ => {
                tracing::warn!("actor name ({actor_type}) does not match message for light");
            }
        }

        Ok(())
    }

    /// Route an esphome motion (`binary_sensor`) reading to the presence actor.
    fn dispatch_esphome_motion(&self, node: &str, payload: &[u8]) -> Result<(), anyhow::Error> {
        let Some(motion) = crate::esphome::parse_binary_state(payload) else {
            tracing::warn!("unrecognised esphome binary state payload for {node}");
            return Ok(());
        };

        let event_id = uuid::Uuid::new_v4();
        let Some(actor_cell) =
            ractor::registry::where_is(TypedActorName::PresenceSensor.to_string())
        else {
            tracing::error!("no presence sensor actor found for esphome motion");
            return Ok(());
        };

        actor_cell.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: presence_sensor::Message::NewEvent(presence_sensor::NewEvent {
                event_id,
                entity: presence_sensor::Entity::Esphome {
                    node: node.to_string(),
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
        let Some(value) = crate::esphome::parse_sensor_state(payload) else {
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
            match ractor::registry::where_is(TypedActorName::EnvironmentSensor.to_string()) {
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
                    serde_json::from_slice::<crate::esphome::EsphomeDiscovery>(&payload)?;
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
                    Some(crate::esphome::EsphomeTarget::Motion { node }) => {
                        self.record_last_seen(&node).await;
                        self.dispatch_esphome_motion(&node, &payload)?
                    }
                    Some(crate::esphome::EsphomeTarget::Sensor { node, object_id }) => {
                        self.record_last_seen(&node).await;
                        self.dispatch_esphome_sensor(&node, &object_id, &payload)?
                    }
                    None => {
                        tracing::warn!("ignoring mqtt packet on unhandled topic: {topic}")
                    }
                }
            }
            MqttTopic::Zigbee2MqttDevice => {
                if let Some(friendly_name) = topic.strip_prefix("zigbee2mqtt/") {
                    self.record_last_seen(friendly_name).await;
                }

                let generic_message =
                    match serde_json::from_slice::<GenericZigbee2MqttMessage>(&payload) {
                        Ok(payload) => payload,
                        Err(e) => {
                            tracing::warn!("unrecognised payload: {payload:?}");
                            return Err(e.into());
                        }
                    };

                let actor_type = generic_message.to_actor_name();
                let actor_name = actor_type.to_string();
                let maybe_actor = ractor::registry::where_is(actor_name);
                let event_id = uuid::Uuid::new_v4();
                tracing::info!("received message for {actor_type}, {generic_message}");

                match maybe_actor {
                    Some(actor_cell) => match actor_type {
                        types::TypedActorName::PresenceSensor => Self::handle_presence_sensor(
                            event_id,
                            actor_type,
                            actor_cell,
                            generic_message,
                        )?,
                        types::TypedActorName::SmartSwitch => Self::handle_smart_switch(
                            event_id,
                            actor_type,
                            actor_cell,
                            generic_message,
                        )?,
                        types::TypedActorName::EnvironmentSensor => {
                            Self::handle_environment_sensor(
                                event_id,
                                actor_type,
                                actor_cell,
                                generic_message,
                            )?
                        }
                        types::TypedActorName::DoorSensor => Self::handle_door_sensor(
                            event_id,
                            actor_type,
                            actor_cell,
                            generic_message,
                        )?,
                        types::TypedActorName::Light => {
                            Self::handle_light(event_id, actor_type, actor_cell, generic_message)?
                        }
                        types::TypedActorName::ControlSwitch => Self::handle_control_switch(
                            event_id,
                            actor_type,
                            actor_cell,
                            generic_message,
                        )?,
                    },
                    None => tracing::error!("no actor found for {actor_type}"),
                }
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
    }
}
