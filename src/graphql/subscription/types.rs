use async_graphql::{SimpleObject, Union};
use uuid::Uuid;

use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBusMessage;

#[derive(SimpleObject)]
pub struct PresenceUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: String,
    /// Human-friendly name, matching the `entities` query.
    pub name: String,
    /// Raw device address the event was emitted for.
    pub sensor: String,
    pub present: bool,
}

#[derive(SimpleObject)]
pub struct DoorUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: String,
    /// Human-friendly name, matching the `entities` query.
    pub name: String,
    /// Raw device address the event was emitted for.
    pub device: String,
    pub open: bool,
}

#[derive(SimpleObject)]
pub struct SwitchUpdate {
    pub event_id: Uuid,
    pub device: String,
    pub action: String,
}

#[derive(SimpleObject)]
pub struct MetricReading {
    pub metric: String,
    pub value: f64,
}

#[derive(SimpleObject)]
pub struct EnvironmentUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: String,
    /// Human-friendly name, matching the `entities` query.
    pub name: String,
    /// Raw device address the event was emitted for.
    pub sensor: String,
    pub readings: Vec<MetricReading>,
}

#[derive(SimpleObject)]
pub struct CronUpdate {
    pub event_id: Uuid,
    pub name: String,
}

#[derive(SimpleObject)]
pub struct LightUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: String,
    /// Human-friendly name, matching the `entities` query.
    pub name: String,
    /// Raw device address the event was emitted for.
    pub device: String,
    pub on: bool,
}

#[derive(SimpleObject)]
pub struct UnifiUpdate {
    pub event_id: Uuid,
    pub mac_address: String,
    pub client: String,
    pub connected: bool,
}

// TODO: friendly names for zigbee devices
#[derive(Union)]
pub enum EventUpdate {
    Presence(PresenceUpdate),
    Door(DoorUpdate),
    Switch(SwitchUpdate),
    Environment(EnvironmentUpdate),
    Cron(CronUpdate),
    Light(LightUpdate),
    Unifi(UnifiUpdate),
}

impl EventUpdate {
    /// Build an update, resolving the raw device address to the same config slug
    /// and human name the `entities` query exposes so clients can correlate the
    /// two by `id`.
    pub fn from_message(msg: EventBusMessage, registry: &DeviceRegistry) -> Self {
        // Reverse the alias map (address -> slug), falling back to the address
        // itself when a device has no configured id, mirroring `entities`.
        let slug = |address: &str| -> String {
            registry
                .id_for_address(address)
                .unwrap_or(address)
                .to_owned()
        };

        match msg {
            EventBusMessage::Presence {
                event_id,
                sensor,
                present,
            } => {
                let id = slug(&sensor);
                let name = registry
                    .presence(&sensor)
                    .map(|s| &s.name)
                    .filter(|n| !n.is_empty())
                    .cloned()
                    .unwrap_or_else(|| id.clone());
                EventUpdate::Presence(PresenceUpdate {
                    event_id,
                    id,
                    name,
                    sensor,
                    present,
                })
            }
            EventBusMessage::Door {
                event_id,
                ieee_addr,
                open,
            } => {
                let device = ieee_addr.to_string();
                let settings = registry.door(&device);
                let id = settings.map(|s| s.id.clone()).unwrap_or_else(|| slug(&device));
                let name = settings.map(|s| s.name.clone()).unwrap_or_else(|| id.clone());
                EventUpdate::Door(DoorUpdate {
                    event_id,
                    id,
                    name,
                    device,
                    open,
                })
            }
            EventBusMessage::SwitchAction {
                event_id,
                ieee_addr,
                action,
            } => EventUpdate::Switch(SwitchUpdate {
                event_id,
                device: ieee_addr.to_string(),
                action,
            }),
            EventBusMessage::Environment {
                event_id,
                sensor,
                readings,
            } => {
                let settings = registry.environment(&sensor);
                let id = settings.map(|s| s.id.clone()).unwrap_or_else(|| slug(&sensor));
                let name = settings.map(|s| s.name.clone()).unwrap_or_else(|| id.clone());
                EventUpdate::Environment(EnvironmentUpdate {
                    event_id,
                    id,
                    name,
                    sensor,
                    readings: readings
                        .into_iter()
                        .map(|reading| MetricReading {
                            metric: format!("{:?}", reading.metric()),
                            value: reading.value(),
                        })
                        .collect(),
                })
            }
            EventBusMessage::Cron { event_id, name } => {
                EventUpdate::Cron(CronUpdate { event_id, name })
            }
            EventBusMessage::Light {
                event_id,
                ieee_addr,
                on,
            } => {
                let device = ieee_addr.to_string();
                let id = slug(&device);
                let name = registry.light(&device).cloned().unwrap_or_else(|| id.clone());
                EventUpdate::Light(LightUpdate {
                    event_id,
                    id,
                    name,
                    device,
                    on,
                })
            }
            EventBusMessage::Unifi {
                event_id,
                mac_address,
                client,
                connected,
            } => EventUpdate::Unifi(UnifiUpdate {
                event_id,
                mac_address,
                client,
                connected,
            }),
        }
    }
}
