use super::{
    alarm::types::AndroidAppAlarmPayload,
    devices::{control_switch, presence_sensor},
    maccas::types::MaccasOfferIngest,
    unifi::types::UnifiWebhookEvent,
};
use crate::{
    actors::{
        alarm::{AlarmActor, AlarmMessage},
        door_sensor, light,
        maccas::{self, MaccasActor},
        smart_switch,
        synergy::{self, SynergyActor},
        temperature_sensor,
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
use tracing::Level;
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
        payload: UnifiWebhookEvent,
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
                        entity: presence_sensor::Entity::AqaraFP1E(aqara_presence),
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

    fn handle_temperature_sensor(
        event_id: Uuid,
        actor_type: TypedActorName,
        actor_cell: ActorCell,
        generic_message: GenericZigbee2MqttMessage,
    ) -> Result<(), anyhow::Error> {
        match generic_message {
            GenericZigbee2MqttMessage::IKEATemperatureSensor(ikea_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: temperature_sensor::Message::NewEvent(temperature_sensor::NewEvent {
                        event_id,
                        entity: temperature_sensor::Entity::IKEAE2112(ikea_temperature_sensor),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(aqara_temperature_sensor) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: temperature_sensor::Message::NewEvent(temperature_sensor::NewEvent {
                        event_id,
                        entity: temperature_sensor::Entity::AqaraWSDCGQ12LM(
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
                    msg: temperature_sensor::Message::NewEvent(temperature_sensor::NewEvent {
                        event_id,
                        entity: temperature_sensor::Entity::LumiWSDCGQ11LM(lumi_temperature_sensor),
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
                    msg: light::LightHandlerMessage::NewEvent(light::NewEvent {
                        event_id,
                        entity: light::Entity::Phillips9290012573A(phillips_light),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::IKEALight(ikea_light) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: light::LightHandlerMessage::NewEvent(light::NewEvent {
                        event_id,
                        entity: light::Entity::IKEALED2201G8(ikea_light),
                    }),
                    options: JobOptions::default(),
                    accepted: None,
                }))?;
            }
            GenericZigbee2MqttMessage::AqaraWhiteLight(aqara_light) => {
                actor_cell.send_message(FactoryMessage::Dispatch(Job {
                    key: (),
                    msg: light::LightHandlerMessage::NewEvent(light::NewEvent {
                        event_id,
                        entity: light::Entity::AqaraT1(aqara_light),
                    }),
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
                        types::TypedActorName::TemperatureSensor => {
                            Self::handle_temperature_sensor(
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
                        actor.send_message(UnifiMessage::ClientConnect {
                            mac_address: mac_address,
                        })?;
                    }
                    "WiFi Client Disconnected" => {
                        actor.send_message(UnifiMessage::ClientDisconnect {
                            mac_address: mac_address,
                        })?;
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
