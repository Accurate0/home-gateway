use super::{
    alarm::types::AndroidAppAlarmPayload,
    devices::{control_switch, plant_sensor, presence_sensor},
    maccas::types::MaccasOfferIngest,
    unifi::types::UnifiWebhookEvent,
};
use crate::{
    actors::{
        alarm::{AlarmActor, AlarmMessage},
        door_sensor, environment_sensor, light,
        maccas::{self, MaccasActor},
        smart_switch,
        synergy::{self, SynergyActor},
        unifi::{UnifiConnectedClientHandler, UnifiMessage, types::Parameters},
    },
    types::SharedActorState,
    zigbee2mqtt::devices::BridgeDevices,
};
use bytes::Bytes;
use ractor::{
    ActorCell, ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use types::{GenericZigbee2MqttMessage, TypedActorName};
use uuid::Uuid;

pub mod spawn;
mod types;

pub enum Message {
    MqttPacket {
        payload: bytes::Bytes,
        topic: String,
    },
    AlarmChangeIngest {
        payload: AndroidAppAlarmPayload,
    },
    MaccasOfferIngest {
        payload: MaccasOfferIngest,
    },
    SynergyDataIngest {
        payload: Bytes,
    },
    UnifiWebhook {
        payload: Box<UnifiWebhookEvent>,
    },
}

pub struct EventHandler {
    shared_actor_state: SharedActorState,
}

impl EventHandler {
    pub const NAME: &str = "event-handler";

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
                    msg: environment_sensor::Message::NewEvent(environment_sensor::NewEvent {
                        event_id,
                        entity: environment_sensor::Entity::IKEAE2112(ikea_temperature_sensor),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(aqara_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: environment_sensor::Message::NewEvent(environment_sensor::NewEvent {
                        event_id,
                        entity: environment_sensor::Entity::AqaraWSDCGQ12LM(
                            aqara_temperature_sensor,
                        ),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }

            GenericZigbee2MqttMessage::LumiTemperatureSensor(lumi_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: environment_sensor::Message::NewEvent(environment_sensor::NewEvent {
                        event_id,
                        entity: environment_sensor::Entity::LumiWSDCGQ11LM(lumi_temperature_sensor),
                    }),
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

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::MqttPacket { payload, topic } if topic == "zigbee2mqtt/bridge/devices" => {
                let devices_payload = serde_json::from_slice::<BridgeDevices>(&payload)?;
                let mut devices_map = self.shared_actor_state.known_devices_map.write().await;
                for device in devices_payload {
                    sqlx::query!(
                            "INSERT INTO known_devices (ieee_addr, name) VALUES ($1, $2) ON CONFLICT (ieee_addr) DO UPDATE SET name = $2",
                                &device.ieee_address,
                                device.friendly_name
                            )
                            .execute(&self.shared_actor_state.db)
                            .await?;
                    devices_map.insert(device.ieee_address, device.friendly_name);
                }
                drop(devices_map)
            }
            Message::MqttPacket { payload, topic } if topic.starts_with("esphome/discover/") => {
                let discovery =
                    serde_json::from_slice::<crate::esphome::EsphomeDiscovery>(&payload)?;
                tracing::info!(
                    "discovered esphome device: {} ({})",
                    discovery.friendly_name,
                    discovery.name
                );

                self.shared_actor_state
                    .known_devices_map
                    .write()
                    .await
                    .insert(discovery.name.clone(), discovery.friendly_name.clone());

                let settings = self.shared_actor_state.settings.load();
                if let Some(presence_settings) = settings.presence_sensors.get(&discovery.name)
                    && presence_settings.sensor_type == crate::settings::PresenceSensorType::Esphome
                {
                    if let Some(motion_entity) = &presence_settings.motion_entity {
                        let topic =
                            crate::esphome::motion_state_topic(&discovery.name, motion_entity);
                        tracing::info!("subscribing to esphome motion topic: {topic}");
                        self.shared_actor_state.mqtt.subscribe(topic).await?;
                    } else {
                        tracing::warn!(
                            "esphome presence sensor {} has no motionEntity configured",
                            discovery.name
                        );
                    }
                }

                if let Some(temperature_settings) =
                    settings.environment_sensors.get(&discovery.name)
                    && temperature_settings.sensor_type
                        == crate::settings::EnvironmentSensorType::Esphome
                {
                    for object_id in crate::esphome::TEMPERATURE_SENSOR_OBJECT_IDS {
                        let topic = crate::esphome::sensor_state_topic(&discovery.name, object_id);
                        tracing::info!("subscribing to esphome sensor topic: {topic}");
                        self.shared_actor_state.mqtt.subscribe(topic).await?;
                    }
                }

                if let Some(plant_settings) = settings.plant_sensors.get(&discovery.name) {
                    // subscribe to each distinct entity referenced by a threshold
                    let entities: std::collections::HashSet<&str> = plant_settings
                        .actions
                        .iter()
                        .map(|a| a.entity.as_str())
                        .collect();
                    for object_id in entities {
                        let topic = crate::esphome::sensor_state_topic(&discovery.name, object_id);
                        tracing::info!("subscribing to esphome plant sensor topic: {topic}");
                        self.shared_actor_state.mqtt.subscribe(topic).await?;
                    }
                }
            }
            Message::MqttPacket { payload, topic }
                if topic.contains("/binary_sensor/") && topic.ends_with("/state") =>
            {
                let Some(node) = topic.split('/').next() else {
                    tracing::warn!("malformed esphome state topic: {topic}");
                    return Ok(());
                };

                let Some(motion) = crate::esphome::parse_binary_state(&payload) else {
                    tracing::warn!("unrecognised esphome binary state payload on {topic}");
                    return Ok(());
                };

                let event_id = uuid::Uuid::new_v4();
                let actor_name = TypedActorName::PresenceSensor.to_string();
                let Some(actor_cell) = ractor::registry::where_is(actor_name) else {
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
            }
            Message::MqttPacket { payload, topic }
                if topic.contains("/sensor/") && topic.ends_with("/state") =>
            {
                let mut segments = topic.split('/');
                let (Some(node), Some(_), Some(object_id)) =
                    (segments.next(), segments.next(), segments.next())
                else {
                    tracing::warn!("malformed esphome sensor topic: {topic}");
                    return Ok(());
                };

                let Some(value) = crate::esphome::parse_sensor_state(&payload) else {
                    tracing::warn!("unrecognised esphome sensor payload on {topic}");
                    return Ok(());
                };

                let event_id = uuid::Uuid::new_v4();
                let settings = self.shared_actor_state.settings.load();

                // a node can be both: the plant actor watches soil moisture for
                // threshold workflows, while the temperature actor stores air
                // temperature/humidity/lux/uv. Dispatch to each independently;
                // each actor ignores object_ids it doesn't handle.
                if settings.plant_sensors.contains_key(node) {
                    match ractor::registry::where_is(
                        plant_sensor::PlantSensorHandler::NAME.to_string(),
                    ) {
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

                if settings.environment_sensors.contains_key(node) {
                    let actor_name = TypedActorName::EnvironmentSensor.to_string();
                    match ractor::registry::where_is(actor_name) {
                        Some(actor_cell) => {
                            actor_cell.send_message(FactoryMessage::Dispatch(Job {
                                key: (),
                                msg: environment_sensor::Message::NewEvent(
                                    environment_sensor::NewEvent {
                                        event_id,
                                        entity: environment_sensor::Entity::Esphome {
                                            node: node.to_string(),
                                            object_id: object_id.to_string(),
                                            value,
                                        },
                                    },
                                ),
                                options: JobOptions::default(),
                                accepted: None,
                            }))?;
                        }
                        None => {
                            tracing::error!("no temperature sensor actor found for esphome sensor")
                        }
                    }
                }
            }
            Message::MqttPacket { payload, .. } => {
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
            Message::MaccasOfferIngest { payload } => {
                tracing::info!(
                    "received maccas offer event for {}",
                    payload.details.short_name
                );

                let maybe_actor = ractor::registry::where_is(MaccasActor::NAME.to_string());
                if let Some(actor) = maybe_actor {
                    actor.send_message(maccas::MaccasMessage::NewOffer(payload))?;
                }
            }
            Message::SynergyDataIngest { payload } => {
                tracing::info!("received synergy event");
                let maybe_actor = ractor::registry::where_is(SynergyActor::NAME.to_string());
                if let Some(actor) = maybe_actor {
                    actor.send_message(synergy::SynergyMessage::NewUpload(payload))?;
                }
            }
            Message::UnifiWebhook { payload } => {
                tracing::info!("received unifi webhook event",);
                let maybe_actor =
                    ractor::registry::where_is(UnifiConnectedClientHandler::NAME.to_string());

                if maybe_actor.is_none() {
                    tracing::warn!("unifi actor not found");
                    return Ok(());
                }

                let actor = maybe_actor.unwrap();

                let mac_address = match payload.parameters {
                    Parameters::Connect(connect_parameters) => connect_parameters.unificlient_mac,
                    Parameters::Disconnect(disconnect_parameters) => {
                        disconnect_parameters.unificlient_mac
                    }
                };

                match payload.name.as_str() {
                    "WiFi Client Connected" => {
                        actor.send_message(UnifiMessage::ClientConnect { mac_address })?;
                    }
                    "WiFi Client Disconnected" => {
                        actor.send_message(UnifiMessage::ClientDisconnect { mac_address })?;
                    }
                    unknown => tracing::warn!("unknown webhook event: {unknown}"),
                }
            }
            Message::AlarmChangeIngest { payload } => {
                tracing::info!("received alarm webhook event");
                let Some(actor) = ractor::registry::where_is(AlarmActor::NAME.to_string()) else {
                    tracing::warn!("alarm actor not found");
                    return Ok(());
                };

                actor.send_message(AlarmMessage::NextAlarm(payload))?;
            }
        };

        Ok(())
    }
}

impl Worker for EventHandler {
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
impl WorkerBuilder<EventHandler, ()> for MqttMessageHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (EventHandler, ()) {
        (
            EventHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
