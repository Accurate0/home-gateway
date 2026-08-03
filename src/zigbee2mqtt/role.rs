use crate::{
    actors::{
        devices::{
            control_switch, control_switch::ControlSwitchHandler, presence_sensor,
            presence_sensor::PresenceSensorHandler,
        },
        door_sensor,
        door_sensor::DoorSensorHandler,
        environment_sensor,
        environment_sensor::EnvironmentSensorHandler,
        light,
        light::LightHandler,
        smart_switch,
        smart_switch::SmartSwitchHandler,
    },
    device_registry::{DeviceRegistry, ZigbeeDevice},
    settings::zigbee_model::{payload_bool, payload_f64, payload_i64, payload_string},
};
use ractor::factory::{FactoryMessage, Job, JobOptions};
use serde_json::{Map, Value};
use uuid::Uuid;

/// One device kind a zigbee payload can feed: whether the device declares it, how
/// to read it off a payload, and how it reaches its actor. `DeviceRegistry::build`
/// has already proven a declared role has a matching profile block, so `extract`
/// reads its mapping directly and only reports whether the payload carried the
/// fields.
pub trait ZigbeeRole: Sized {
    type Message: ractor::Message;

    const ACTOR: &'static str;

    fn declared(devices: &DeviceRegistry, address: &str) -> bool;

    fn extract(
        device: &ZigbeeDevice,
        friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self>;

    fn into_message(self, event_id: Uuid) -> Self::Message;
}

impl ZigbeeRole for door_sensor::Entity {
    type Message = door_sensor::Message;

    const ACTOR: &'static str = DoorSensorHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.door(address).is_some()
    }

    fn extract(
        device: &ZigbeeDevice,
        friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let fields = device.profile.door.as_ref()?;
        let address = &device.address;

        let Some(contact) = payload_bool(payload, &fields.contact) else {
            tracing::info!("skipping door reading for {address}: no contact in payload");
            return None;
        };

        let Some(battery) = battery(device, payload) else {
            tracing::info!("skipping door reading for {address}: no battery in payload");
            return None;
        };

        Some(door_sensor::Entity::Zigbee {
            address: address.clone(),
            friendly_name: friendly_name.to_owned(),
            contact,
            battery,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        door_sensor::Message::NewEvent(door_sensor::NewEvent {
            event_id,
            entity: self,
        })
    }
}

impl ZigbeeRole for environment_sensor::Entity {
    type Message = environment_sensor::Message;

    const ACTOR: &'static str = EnvironmentSensorHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.environment(address).is_some()
    }

    fn extract(
        device: &ZigbeeDevice,
        friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let mapping = device.profile.environment.as_ref()?;
        let address = &device.address;
        let mut readings = Vec::new();

        for (metric, key) in mapping {
            match payload_f64(payload, key) {
                Some(value) => readings.push((*metric, value)),
                None => tracing::info!("no {key} in payload for {address}"),
            }
        }

        if readings.is_empty() {
            tracing::info!("skipping environment reading for {address}: no metrics in payload");
            return None;
        }

        Some(environment_sensor::Entity::Zigbee {
            address: address.clone(),
            friendly_name: friendly_name.to_owned(),
            readings,
            battery: battery(device, payload),
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        environment_sensor::Message::NewEvent(Box::new(environment_sensor::NewEvent {
            event_id,
            entity: self,
        }))
    }
}

impl ZigbeeRole for light::Entity {
    type Message = light::LightHandlerMessage;

    const ACTOR: &'static str = LightHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.light(address).is_some()
    }

    fn extract(
        device: &ZigbeeDevice,
        _friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let fields = device.profile.light.as_ref()?;
        let address = &device.address;

        let Some(state) = payload_string(payload, &fields.state) else {
            tracing::info!("skipping light reading for {address}: no state in payload");
            return None;
        };

        Some(light::Entity::Zigbee {
            address: address.clone(),
            state,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        light::LightHandlerMessage::NewEvent(Box::new(light::NewEvent {
            event_id,
            entity: self,
        }))
    }
}

impl ZigbeeRole for smart_switch::Entity {
    type Message = smart_switch::Message;

    const ACTOR: &'static str = SmartSwitchHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.smart_switch(address).is_some()
    }

    fn extract(
        device: &ZigbeeDevice,
        friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let fields = device.profile.smart_switch.as_ref()?;
        let address = &device.address;

        let voltage = payload_i64(payload, &fields.voltage);
        let power = payload_i64(payload, &fields.power);
        let current = payload_f64(payload, &fields.current);
        let energy = payload_f64(payload, &fields.energy);

        let (Some(voltage), Some(power), Some(current), Some(energy)) =
            (voltage, power, current, energy)
        else {
            tracing::info!("skipping smart switch reading for {address}: partial payload");
            return None;
        };

        Some(smart_switch::Entity::Zigbee {
            address: address.clone(),
            friendly_name: friendly_name.to_owned(),
            voltage,
            power,
            current,
            energy,
            state: fields
                .state
                .as_ref()
                .and_then(|key| payload_string(payload, key)),
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        smart_switch::Message::NewEvent(smart_switch::NewEvent {
            event_id,
            entity: self,
        })
    }
}

impl ZigbeeRole for presence_sensor::Entity {
    type Message = presence_sensor::Message;

    const ACTOR: &'static str = PresenceSensorHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.presence(address).is_some()
    }

    fn extract(
        device: &ZigbeeDevice,
        _friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let fields = device.profile.presence.as_ref()?;
        let address = &device.address;

        let Some(presence) = payload_bool(payload, &fields.presence) else {
            tracing::info!("skipping presence reading for {address}: no presence in payload");
            return None;
        };

        Some(presence_sensor::Entity::Zigbee {
            address: address.clone(),
            presence,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        presence_sensor::Message::NewEvent(presence_sensor::NewEvent {
            event_id,
            entity: self,
        })
    }
}

impl ZigbeeRole for control_switch::Entity {
    type Message = control_switch::ControlSwitchMessage;

    const ACTOR: &'static str = ControlSwitchHandler::NAME;
    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.control_switch(address)
    }

    fn extract(
        device: &ZigbeeDevice,
        _friendly_name: &str,
        payload: &Map<String, Value>,
    ) -> Option<Self> {
        let fields = device.profile.control_switch.as_ref()?;
        let address = &device.address;

        let action = payload_string(payload, &fields.action).filter(|action| !action.is_empty());

        let Some(action) = action else {
            tracing::info!("skipping control switch reading for {address}: no action in payload");
            return None;
        };

        Some(control_switch::Entity::Zigbee {
            address: address.clone(),
            action,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        control_switch::ControlSwitchMessage::NewEvent(control_switch::NewEvent {
            event_id,
            entity: self,
        })
    }
}

/// Extract one role and hand it to its actor. Roles the device does not declare
/// are skipped without touching the payload.
pub fn run<R: ZigbeeRole>(
    event_id: Uuid,
    devices: &DeviceRegistry,
    device: &ZigbeeDevice,
    friendly_name: &str,
    payload: &Map<String, Value>,
) {
    if !R::declared(devices, &device.address) {
        return;
    }

    let Some(entity) = R::extract(device, friendly_name, payload) else {
        return;
    };

    let Some(actor) = ractor::registry::where_is(R::ACTOR) else {
        tracing::error!("no {} actor found for zigbee event", R::ACTOR);
        return;
    };

    if let Err(e) = actor.send_message(FactoryMessage::<(), R::Message>::Dispatch(Job {
        key: (),
        msg: entity.into_message(event_id),
        options: JobOptions::default(),
        accepted: None,
    })) {
        tracing::error!("failed to dispatch zigbee event to {}: {e}", R::ACTOR);
    }
}

/// Battery percentage, shared by the door and environment rows that store it
/// alongside their own readings and by the battery actor.
pub fn battery(device: &ZigbeeDevice, payload: &Map<String, Value>) -> Option<i64> {
    let key = device.profile.battery.as_ref()?;

    payload_i64(payload, key)
}

/// Payload fields with no column of their own, bound for the generic metric sink.
pub fn metrics(
    device: &ZigbeeDevice,
    payload: &Map<String, Value>,
) -> Vec<(String, crate::device_metric::MetricValue)> {
    let mut out = Vec::new();

    for (name, field) in &device.profile.metrics {
        match crate::settings::zigbee_model::extract_metric(payload, field) {
            Some(value) => out.push((name.clone(), value)),
            None => tracing::info!("no {} in payload for {}", field.key, device.address),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        device_metric::MetricValue,
        settings::{
            Metric,
            zigbee_model::{RawZigbeeModelProfile, ZigbeeModelProfile},
        },
    };
    use std::sync::Arc;

    fn device(slug: &str, profile_yaml: &str) -> ZigbeeDevice {
        let raw = serde_yaml::from_str::<RawZigbeeModelProfile>(profile_yaml).expect("profile");
        let profile = ZigbeeModelProfile::resolve(slug.to_owned(), raw).expect("resolve");

        ZigbeeDevice {
            id: "test-device".to_owned(),
            address: "0xabc".to_owned(),
            profile: Arc::new(profile),
        }
    }

    fn payload(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("payload")
    }

    #[test]
    fn extracts_an_aqara_door_payload() {
        let device = device(
            "aqara_mccgq12lm",
            "battery: battery\ndoor: [contact]\nmetrics:\n  device_temperature: { key: device_temperature, type: float }\n  voltage: { key: voltage, type: int }",
        );
        let payload = payload(
            r#"{"contact":false,"battery":97,"device_temperature":21,"voltage":3005,"linkquality":72}"#,
        );

        let Some(door_sensor::Entity::Zigbee {
            contact,
            battery: level,
            friendly_name,
            ..
        }) = <door_sensor::Entity as ZigbeeRole>::extract(&device, "front-door", &payload)
        else {
            panic!("expected a door reading");
        };

        assert!(!contact);
        assert_eq!(level, 97);
        assert_eq!(friendly_name, "front-door");
        assert_eq!(battery(&device, &payload), Some(97));

        let metrics = metrics(&device, &payload);
        assert!(metrics.contains(&("voltage".to_owned(), MetricValue::Numeric(3005.0))));
        assert!(metrics.contains(&("device_temperature".to_owned(), MetricValue::Numeric(21.0))));
    }

    #[test]
    fn a_door_payload_without_contact_yields_nothing() {
        let device = device(
            "aqara_mccgq12lm",
            "battery: battery\ndoor: [contact]",
        );

        assert!(
            <door_sensor::Entity as ZigbeeRole>::extract(&device, "front-door", &payload(r#"{"battery":97}"#)).is_none(),
            "a payload without contact is not a door event"
        );
    }

    #[test]
    fn extracts_an_environment_payload() {
        let device = device(
            "lumi_wsdcgq11lm",
            "battery: battery\nenvironment: [temperature, humidity, pressure]",
        );
        let payload = payload(r#"{"temperature":18.4,"humidity":61,"pressure":1012,"battery":88}"#);

        let Some(environment_sensor::Entity::Zigbee {
            readings,
            battery: level,
            ..
        }) = <environment_sensor::Entity as ZigbeeRole>::extract(&device, "outdoor", &payload)
        else {
            panic!("expected an environment reading");
        };

        assert!(readings.contains(&(Metric::Temperature, 18.4)));
        assert!(readings.contains(&(Metric::Humidity, 61.0)));
        assert!(readings.contains(&(Metric::Pressure, 1012.0)));
        assert_eq!(level, Some(88));
    }

    #[test]
    fn extracts_an_fp1e_presence_payload_with_text_metrics() {
        let device = device(
            "aqara_fp1e",
            "presence: [presence]\nmetrics:\n  target_distance: { key: target_distance, type: float }\n  movement: { key: movement, type: string }",
        );
        let payload = payload(r#"{"presence":true,"target_distance":1.4,"movement":"approach"}"#);

        let Some(presence_sensor::Entity::Zigbee { presence, .. }) =
            <presence_sensor::Entity as ZigbeeRole>::extract(&device, "closet-presence", &payload)
        else {
            panic!("expected a presence reading");
        };

        assert!(presence);

        let metrics = metrics(&device, &payload);
        assert!(metrics.contains(&("target_distance".to_owned(), MetricValue::Numeric(1.4))));
        assert!(metrics.contains(&(
            "movement".to_owned(),
            MetricValue::Text("approach".to_owned())
        )));
    }

    #[test]
    fn extracts_a_smart_switch_payload() {
        let device = device(
            "ts011f_plug",
            "smart_switch: [state, voltage, power, current, energy]",
        );

        let Some(smart_switch::Entity::Zigbee {
            voltage,
            power,
            current,
            energy,
            state,
            ..
        }) = <smart_switch::Entity as ZigbeeRole>::extract(
            &device,
            "living-room-lamp",
            &payload(
                r#"{"state":"ON","voltage":244,"power":12,"current":0.05,"energy":13.37,"child_lock":"UNLOCK"}"#,
            ),
        )
        else {
            panic!("expected a smart switch reading");
        };

        assert_eq!(voltage, 244);
        assert_eq!(power, 12);
        assert_eq!(current, 0.05);
        assert_eq!(energy, 13.37);
        assert_eq!(state.as_deref(), Some("ON"));
    }

    #[test]
    fn a_partial_smart_switch_payload_yields_nothing() {
        let device = device(
            "ts011f_plug",
            "smart_switch: [state, voltage, power, current, energy]",
        );

        assert!(
            <smart_switch::Entity as ZigbeeRole>::extract(
                &device,
                "living-room-lamp",
                &payload(r#"{"state":"ON","voltage":244}"#)
            )
            .is_none(),
            "a partial payload is not a smart switch event"
        );
    }

    #[test]
    fn an_empty_control_switch_action_is_dropped() {
        let device = device(
            "aqara_wxkg11lm",
            "battery: battery\ncontrol_switch: [action]",
        );

        assert!(
            <control_switch::Entity as ZigbeeRole>::extract(
                &device,
                "small-switch",
                &payload(r#"{"action":"","battery":91}"#)
            )
            .is_none(),
            "an empty action is not an event"
        );

        let Some(control_switch::Entity::Zigbee { action, .. }) = <control_switch::Entity as ZigbeeRole>::extract(
            &device,
            "small-switch",
            &payload(r#"{"action":"single","battery":91}"#),
        ) else {
            panic!("expected a control switch reading");
        };

        assert_eq!(action, "single");
    }

}
