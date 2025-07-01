use crate::{
    actors::{
        devices::unifi::{self, UnifiConnectedClientHandler},
        door_sensor, light, smart_switch, temperature_sensor,
    },
    maccas::MaccasOfferIngest,
    types::SharedActorState,
    unifi::types::UnifiConnectedClients,
    zigbee2mqtt::devices::BridgeDevices,
};
use ractor::{
    ActorCell, ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use serde::{Deserialize, Serialize};
use types::{GenericZigbee2MqttMessage, TypedActorName};
use uuid::Uuid;

pub mod spawn;
mod types;

pub enum Message {
    MqttPacket {
        payload: bytes::Bytes,
        topic: String,
    },
    UnifiClients {
        payload: UnifiConnectedClients,
    },
    MaccasOfferIngest {
        payload: MaccasOfferIngest,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "event_type", rename_all = "lowercase")]
pub enum EventType {
    Mqtt,
    Unifi,
}

pub struct EventHandler {
    shared_actor_state: SharedActorState,
}

impl EventHandler {
    pub const NAME: &str = "event-handler";

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
                    msg: light::Message::NewEvent(light::NewEvent {
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
                    msg: light::Message::NewEvent(light::NewEvent {
                        event_id,
                        entity: light::Entity::IKEALED2201G8(ikea_light),
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
                    devices_map.insert(device.ieee_address);
                }
                drop(devices_map)
            }
            Message::MqttPacket { payload, .. } => {
                let generic_message =
                    serde_json::from_slice::<GenericZigbee2MqttMessage>(&payload)?;
                let actor_type = generic_message.to_actor_name();
                let actor_name = actor_type.to_string();
                let maybe_actor = ractor::registry::where_is(actor_name);
                let event_id: Uuid = sqlx::query_scalar!(
                    r#"
                    INSERT INTO events (raw_data, event_type)
                    VALUES ((REPLACE($1::text, '\u0000', ''))::jsonb, $2)
                    RETURNING id "#,
                    serde_json::to_string(&generic_message)?,
                    EventType::Mqtt as EventType
                )
                .fetch_one(&self.shared_actor_state.db)
                .await?;
                tracing::info!("received message for {actor_type}, {generic_message}");

                match maybe_actor {
                    Some(actor_cell) => match actor_type {
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
                    },
                    None => tracing::error!("no actor found for {actor_type}"),
                }
            }
            Message::UnifiClients { payload } => {
                let event_id: Uuid = sqlx::query_scalar!(
                    r#"
                    INSERT INTO events (raw_data, event_type)
                    VALUES ($1, $2)
                    RETURNING id
                    "#,
                    serde_json::to_value(&payload)?,
                    EventType::Unifi as EventType
                )
                .fetch_one(&self.shared_actor_state.db)
                .await?;

                let maybe_actor =
                    ractor::registry::where_is(UnifiConnectedClientHandler::NAME.to_string());

                if let Some(actor) = maybe_actor {
                    actor.send_message(unifi::Message::NewEvent {
                        clients: payload,
                        event_id,
                    })?;
                }

                tracing::info!("received message for unifi");
            }
            Message::MaccasOfferIngest { payload } => {
                tracing::info!(
                    "received maccas offer event for {}",
                    payload.details.short_name
                )
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
