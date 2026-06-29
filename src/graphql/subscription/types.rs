use async_graphql::{SimpleObject, Union};
use uuid::Uuid;

use crate::event_bus::EventBusMessage;

#[derive(SimpleObject)]
pub struct PresenceUpdate {
    pub event_id: Uuid,
    pub sensor: String,
    pub present: bool,
}

#[derive(SimpleObject)]
pub struct DoorUpdate {
    pub event_id: Uuid,
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

impl From<EventBusMessage> for EventUpdate {
    fn from(msg: EventBusMessage) -> Self {
        match msg {
            EventBusMessage::Presence {
                event_id,
                sensor,
                present,
            } => EventUpdate::Presence(PresenceUpdate {
                event_id,
                sensor,
                present,
            }),
            EventBusMessage::Door {
                event_id,
                ieee_addr,
                open,
            } => EventUpdate::Door(DoorUpdate {
                event_id,
                device: ieee_addr.to_string(),
                open,
            }),
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
            } => EventUpdate::Environment(EnvironmentUpdate {
                event_id,
                sensor,
                readings: readings
                    .into_iter()
                    .map(|reading| MetricReading {
                        metric: format!("{:?}", reading.metric()),
                        value: reading.value(),
                    })
                    .collect(),
            }),
            EventBusMessage::Cron { event_id, name } => {
                EventUpdate::Cron(CronUpdate { event_id, name })
            }
            EventBusMessage::Light {
                event_id,
                ieee_addr,
                on,
            } => EventUpdate::Light(LightUpdate {
                event_id,
                device: ieee_addr.to_string(),
                on,
            }),
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
