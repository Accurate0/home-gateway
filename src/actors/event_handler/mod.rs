use super::{
    maccas::types::MaccasOfferIngest, synergy::types::S3BucketEvent,
    unifi::types::UnifiWebhookEvents,
};
use crate::{
    actors::{
        door_sensor, light,
        maccas::{self, MaccasActor},
        smart_switch,
        synergy::{self, SynergyActor},
        temperature_sensor,
        unifi::{UnifiConnectedClientHandler, UnifiMessage},
    },
    types::SharedActorState,
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
    MaccasOfferIngest {
        payload: MaccasOfferIngest,
    },
    SynergyDataIngest {
        payload: S3BucketEvent,
    },
    UnifiWebhook {
        payload: UnifiWebhookEvents,
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

    #[tracing::instrument(name = "handle_smart_switch", skip_all)]
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

    #[tracing::instrument(name = "handle_temperature_sensor", skip_all)]
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

    #[tracing::instrument(name = "handle_door_sensor", skip_all)]
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

    #[tracing::instrument(name = "handle_light", skip_all)]
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

    #[tracing::instrument(name = "event-handler", skip(self, message))]
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
                tracing::info!("received synergy event for {}", payload.key);

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

                for event in payload.events {
                    match event.id.as_str() {
                        "event.client_connected" => {
                            actor.send_message(UnifiMessage::ClientConnect {
                                mac_address: event.scope.client_mac_address,
                            })?;
                        }
                        "event.client_disconnected" => {
                            actor.send_message(UnifiMessage::ClientDisconnect {
                                mac_address: event.scope.client_mac_address,
                            })?;
                        }
                        unknown => tracing::warn!("unknown webhook event: {unknown}"),
                    }
                }
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
