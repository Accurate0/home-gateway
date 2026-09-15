use crate::actors::system::rpc;
use crate::{
    actors::devices::{
        control_switch, control_switch::ControlSwitchHandler, door_sensor,
        door_sensor::DoorSensorHandler, environment_sensor,
        environment_sensor::EnvironmentSensorHandler, light, light::LightHandler, presence_sensor,
        presence_sensor::PresenceSensorHandler, smart_switch, smart_switch::SmartSwitchHandler,
    },
    device_registry::{DeviceRegistry, ZigbeeDevice},
    repo::light::LightAttributes,
    settings::ZigbeeReading,
};

use uuid::Uuid;

pub trait ZigbeeRole: Sized {
    type Message: ractor::Message;

    const ACTOR: &'static str;

    fn declared(devices: &DeviceRegistry, address: &str) -> bool;

    fn extract(device: &ZigbeeDevice, friendly_name: &str, reading: &ZigbeeReading)
    -> Option<Self>;

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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;

        let Some(contact) = reading.door.as_ref().and_then(|fields| fields.contact) else {
            tracing::info!("skipping door reading for {address}: no contact in payload");
            return None;
        };

        let Some(battery) = reading.battery else {
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
            traceparent: crate::tracing_context::inject_current(),
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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;
        let declared = &device.profile.environment;

        let Some(reported) = reading.environment.as_ref() else {
            tracing::info!("skipping environment reading for {address}: no environment in payload");
            return None;
        };

        for metric in reported.keys().filter(|metric| !declared.contains(metric)) {
            tracing::warn!(
                "zigbee model {} reported undeclared environment metric {metric:?} for {address}",
                device.profile.slug
            );
        }

        let mut readings = Vec::new();

        for metric in declared {
            match reported.get(metric) {
                Some(value) => readings.push((*metric, *value)),
                None => tracing::info!("no {metric:?} in payload for {address}"),
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
            battery: reading.battery,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        environment_sensor::Message::NewEvent(Box::new(environment_sensor::NewEvent {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;

        let Some(fields) = reading.light.as_ref() else {
            tracing::info!("skipping light reading for {address}: no light in payload");
            return None;
        };

        let Some(state) = fields.state.clone() else {
            tracing::info!("skipping light reading for {address}: no state in payload");
            return None;
        };

        Some(light::Entity::Zigbee {
            address: address.clone(),
            attributes: LightAttributes {
                state: Some(state),
                brightness: fields.brightness,
                colour_temp: fields.color_temp,
                colour: fields.color.as_ref().and_then(light::colour_hex),
            },
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        light::LightHandlerMessage::NewEvent(Box::new(light::NewEvent {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;

        let Some(fields) = reading.smart_switch.as_ref() else {
            tracing::info!(
                "skipping smart switch reading for {address}: no smart_switch in payload"
            );
            return None;
        };

        let (Some(voltage), Some(power), Some(current), Some(energy)) =
            (fields.voltage, fields.power, fields.current, fields.energy)
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
            state: fields.state.clone(),
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        smart_switch::Message::NewEvent(smart_switch::NewEvent {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;

        let Some(presence) = reading.presence.as_ref().and_then(|fields| fields.presence) else {
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
            traceparent: crate::tracing_context::inject_current(),
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
        reading: &ZigbeeReading,
    ) -> Option<Self> {
        let address = &device.address;

        let action = reading
            .control_switch
            .as_ref()
            .and_then(|fields| fields.action.clone())
            .filter(|action| !action.is_empty());

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
            traceparent: crate::tracing_context::inject_current(),
            entity: self,
        })
    }
}

pub fn run<R: ZigbeeRole>(
    event_id: Uuid,
    devices: &DeviceRegistry,
    device: &ZigbeeDevice,
    friendly_name: &str,
    reading: &ZigbeeReading,
) {
    if !R::declared(devices, &device.address) {
        return;
    }

    let Some(entity) = R::extract(device, friendly_name, reading) else {
        return;
    };

    if let Err(e) = rpc::cast_factory(R::ACTOR, entity.into_message(event_id)) {
        tracing::error!("failed to dispatch zigbee event to {}: {e}", R::ACTOR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        device_metric::MetricValue,
        settings::{LuaSettings, Metric, load_zigbee_models},
    };
    use serde_json::{Map, Value};
    use std::collections::BTreeMap;

    fn device(slug: &str, source: &str) -> ZigbeeDevice {
        let sources = BTreeMap::from([(slug.to_owned(), source.to_owned())]);
        let models = load_zigbee_models(&sources, &LuaSettings::default()).expect("models");

        ZigbeeDevice {
            id: "test-device".to_owned(),
            address: "0xabc".to_owned(),
            profile: models[slug].clone(),
        }
    }

    fn reading(device: &ZigbeeDevice, json: &str) -> ZigbeeReading {
        let payload: Map<String, Value> = serde_json::from_str(json).expect("payload");

        device.profile.decode(&payload).expect("decode")
    }

    fn metrics(reading: ZigbeeReading) -> Vec<(String, MetricValue)> {
        reading
            .metrics
            .into_iter()
            .map(|(name, value)| (name, value.into()))
            .collect()
    }

    const AQARA_DOOR: &str = include_str!("../../../config/lua/zigbee/aqara_mccgq12lm.lua");
    const LUMI_ENVIRONMENT: &str = include_str!("../../../config/lua/zigbee/lumi_wsdcgq11lm.lua");
    const AQARA_FP1E: &str = include_str!("../../../config/lua/zigbee/aqara_fp1e.lua");
    const TS011F_PLUG: &str = include_str!("../../../config/lua/zigbee/ts011f_plug.lua");
    const AQARA_SWITCH: &str = include_str!("../../../config/lua/zigbee/aqara_wxkg11lm.lua");
    const AQARA_T1: &str = include_str!("../../../config/lua/zigbee/aqara_t1.lua");

    #[test]
    fn extracts_an_aqara_door_payload() {
        let device = device("aqara_mccgq12lm", AQARA_DOOR);
        let reading = reading(
            &device,
            r#"{"contact":false,"battery":97,"device_temperature":21,"voltage":3005,"linkquality":72}"#,
        );

        let Some(door_sensor::Entity::Zigbee {
            contact,
            battery: level,
            friendly_name,
            ..
        }) = <door_sensor::Entity as ZigbeeRole>::extract(&device, "front-door", &reading)
        else {
            panic!("expected a door reading");
        };

        assert!(!contact);
        assert_eq!(level, 97);
        assert_eq!(friendly_name, "front-door");
        assert_eq!(reading.battery, Some(97));

        let metrics = metrics(reading);
        assert!(metrics.contains(&("voltage".to_owned(), MetricValue::Numeric(3005.0))));
        assert!(metrics.contains(&("device_temperature".to_owned(), MetricValue::Numeric(21.0))));
    }

    #[test]
    fn a_door_payload_without_contact_yields_nothing() {
        let device = device("aqara_mccgq12lm", AQARA_DOOR);

        assert!(
            <door_sensor::Entity as ZigbeeRole>::extract(
                &device,
                "front-door",
                &reading(&device, r#"{"battery":97}"#)
            )
            .is_none(),
            "a payload without contact is not a door event"
        );
    }

    #[test]
    fn extracts_an_environment_payload() {
        let device = device("lumi_wsdcgq11lm", LUMI_ENVIRONMENT);
        let reading = reading(
            &device,
            r#"{"temperature":18.4,"humidity":61,"pressure":1012,"battery":88}"#,
        );

        let Some(environment_sensor::Entity::Zigbee {
            readings,
            battery: level,
            ..
        }) = <environment_sensor::Entity as ZigbeeRole>::extract(&device, "outdoor", &reading)
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
        let device = device("aqara_fp1e", AQARA_FP1E);
        let reading = reading(
            &device,
            r#"{"presence":true,"target_distance":1.4,"movement":"approach"}"#,
        );

        let Some(presence_sensor::Entity::Zigbee { presence, .. }) =
            <presence_sensor::Entity as ZigbeeRole>::extract(&device, "closet-presence", &reading)
        else {
            panic!("expected a presence reading");
        };

        assert!(presence);

        let metrics = metrics(reading);
        assert!(metrics.contains(&("target_distance".to_owned(), MetricValue::Numeric(1.4))));
        assert!(metrics.contains(&(
            "movement".to_owned(),
            MetricValue::Text("approach".to_owned())
        )));
    }

    #[test]
    fn extracts_a_smart_switch_payload() {
        let device = device("ts011f_plug", TS011F_PLUG);

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
            &reading(
                &device,
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
        let device = device("ts011f_plug", TS011F_PLUG);

        assert!(
            <smart_switch::Entity as ZigbeeRole>::extract(
                &device,
                "living-room-lamp",
                &reading(&device, r#"{"state":"ON","voltage":244}"#)
            )
            .is_none(),
            "a partial payload is not a smart switch event"
        );
    }

    #[test]
    fn extracts_a_light_payload_with_colour() {
        let device = device("aqara_t1", AQARA_T1);

        let Some(light::Entity::Zigbee { attributes, .. }) = <light::Entity as ZigbeeRole>::extract(
            &device,
            "closet-light",
            &reading(
                &device,
                r##"{"state":"ON","brightness":120,"color_temp":370,"color":{"hex":"#FF8800"}}"##,
            ),
        ) else {
            panic!("expected a light reading");
        };

        assert_eq!(attributes.state.as_deref(), Some("ON"));
        assert_eq!(attributes.brightness, Some(120));
        assert_eq!(attributes.colour_temp, Some(370));
        assert!(attributes.colour.is_some());
    }

    #[test]
    fn an_empty_control_switch_action_is_dropped() {
        let device = device("aqara_wxkg11lm", AQARA_SWITCH);

        assert!(
            <control_switch::Entity as ZigbeeRole>::extract(
                &device,
                "small-switch",
                &reading(&device, r#"{"action":"","battery":91}"#)
            )
            .is_none(),
            "an empty action is not an event"
        );

        let Some(control_switch::Entity::Zigbee { action, .. }) =
            <control_switch::Entity as ZigbeeRole>::extract(
                &device,
                "small-switch",
                &reading(&device, r#"{"action":"single","battery":91}"#),
            )
        else {
            panic!("expected a control switch reading");
        };

        assert_eq!(action, "single");
    }
}
