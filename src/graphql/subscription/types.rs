use async_graphql::{ComplexObject, ID, SimpleObject, Union};
use uuid::Uuid;

use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBusMessage;
use crate::mode::Mode;

#[derive(SimpleObject)]
pub struct PresenceUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: ID,
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
    pub id: ID,
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
    pub id: ID,
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
pub struct SunUpdate {
    pub event_id: Uuid,
    pub transition: String,
}

#[derive(SimpleObject)]
pub struct LightUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: ID,
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

#[derive(SimpleObject)]
pub struct ModeUpdate {
    pub event_id: Uuid,
    pub mode: Mode,
    pub active: bool,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct HomeAssistantUpdate {
    pub event_id: Uuid,
    pub entity_id: String,
    pub state: String,
}

#[ComplexObject]
impl HomeAssistantUpdate {
    async fn id(&self) -> ID {
        ID(self.event_id.to_string())
    }
}

#[derive(SimpleObject)]
pub struct WoolworthsUpdate {
    pub event_id: Uuid,
    pub product_id: i64,
    pub name: String,
    pub old_price: f64,
    pub new_price: f64,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DeviceBatteryUpdate {
    pub event_id: Uuid,
    /// Config slug, matching the `id` from the `entities` query.
    pub id: ID,
    pub name: String,
    pub kind: String,
    pub battery_voltage: f64,
}

#[ComplexObject]
impl DeviceBatteryUpdate {
    async fn battery_percentage(&self) -> f64 {
        crate::battery::voltage_to_percentage(self.battery_voltage)
    }
}

// TODO: friendly names for zigbee devices
#[derive(Union)]
pub enum EventUpdate {
    Presence(PresenceUpdate),
    Door(DoorUpdate),
    Switch(SwitchUpdate),
    Environment(EnvironmentUpdate),
    Cron(CronUpdate),
    Sun(SunUpdate),
    Light(LightUpdate),
    Unifi(UnifiUpdate),
    Mode(ModeUpdate),
    HomeAssistant(HomeAssistantUpdate),
    Woolworths(WoolworthsUpdate),
    DeviceBattery(DeviceBatteryUpdate),
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
                    id: ID(id),
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
                let id = settings
                    .map(|s| s.id.clone())
                    .unwrap_or_else(|| slug(&device));
                let name = settings
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| id.clone());
                EventUpdate::Door(DoorUpdate {
                    event_id,
                    id: ID(id),
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
                let id = settings
                    .map(|s| s.id.clone())
                    .unwrap_or_else(|| slug(&sensor));
                let name = settings
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| id.clone());
                EventUpdate::Environment(EnvironmentUpdate {
                    event_id,
                    id: ID(id),
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
            EventBusMessage::Sun {
                event_id,
                transition,
                ..
            } => EventUpdate::Sun(SunUpdate {
                event_id,
                transition: format!("{transition:?}"),
            }),
            EventBusMessage::Light {
                event_id,
                ieee_addr,
                on,
            } => {
                let device = ieee_addr.to_string();
                let id = slug(&device);
                let name = registry
                    .light(&device)
                    .cloned()
                    .unwrap_or_else(|| id.clone());
                EventUpdate::Light(LightUpdate {
                    event_id,
                    id: ID(id),
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
            EventBusMessage::Mode {
                event_id,
                mode,
                active,
            } => EventUpdate::Mode(ModeUpdate {
                event_id,
                mode,
                active,
            }),
            EventBusMessage::HomeAssistant {
                event_id,
                entity_id,
                state,
            } => EventUpdate::HomeAssistant(HomeAssistantUpdate {
                event_id,
                entity_id,
                state,
            }),
            EventBusMessage::Woolworths {
                event_id,
                product_id,
                name,
                old_price,
                new_price,
            } => EventUpdate::Woolworths(WoolworthsUpdate {
                event_id,
                product_id,
                name,
                old_price,
                new_price,
            }),
            EventBusMessage::DeviceBattery {
                event_id,
                device_id,
                kind,
                name,
                battery_voltage,
            } => EventUpdate::DeviceBattery(DeviceBatteryUpdate {
                event_id,
                id: ID(slug(&device_id)),
                name,
                kind,
                battery_voltage,
            }),
        }
    }
}
