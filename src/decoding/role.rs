use crate::actors::system::rpc;
use crate::{
    actors::devices::{
        control_switch, control_switch::ControlSwitchHandler, door_sensor,
        door_sensor::DoorSensorHandler, environment_sensor,
        environment_sensor::EnvironmentSensorHandler, light, light::LightHandler, media_player,
        media_player::MediaPlayerHandler, plant_sensor, plant_sensor::PlantSensorHandler,
        presence_sensor, presence_sensor::PresenceSensorHandler, robot_vacuum,
        robot_vacuum::RobotVacuumHandler, smart_switch, smart_switch::SmartSwitchHandler,
    },
    device_registry::DeviceRegistry,
    repo::light::LightAttributes,
};

use super::decoded_device::DecodedDevice;
use super::reading::DeviceReading;

use uuid::Uuid;

pub trait DecodedRole: Sized {
    type Message: ractor::Message;

    const ACTOR: &'static str;

    fn present(reading: &DeviceReading) -> bool;

    fn declared(devices: &DeviceRegistry, address: &str) -> bool;

    fn extract(
        device: &DecodedDevice,
        friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self>;

    fn into_message(self, event_id: Uuid) -> Self::Message;
}

impl DecodedRole for door_sensor::Entity {
    type Message = door_sensor::Message;

    const ACTOR: &'static str = DoorSensorHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.door.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.door(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let Some(contact) = reading.door.as_ref()?.contact else {
            tracing::info!("skipping door reading for {address}: no contact in payload");
            return None;
        };

        Some(door_sensor::Entity::Decoded {
            address: address.clone(),
            friendly_name: friendly_name.to_owned(),
            contact,
            battery: reading.battery,
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

impl DecodedRole for environment_sensor::Entity {
    type Message = environment_sensor::Message;

    const ACTOR: &'static str = EnvironmentSensorHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.environment.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.environment(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;
        let declared = &device.profile.environment;

        let reported = reading.environment.as_ref()?;

        for metric in reported.keys().filter(|metric| !declared.contains(metric)) {
            tracing::warn!(
                "{} model {} reported undeclared environment metric {metric:?} for {address}",
                device.profile.source(),
                device.profile.slug
            );
        }

        let mut readings = Vec::new();

        for metric in declared {
            match reported.get(metric) {
                Some(value) => readings.push((*metric, *value)),
                None => tracing::debug!("no {metric:?} in reading for {address}"),
            }
        }

        if readings.is_empty() {
            tracing::info!("skipping environment reading for {address}: no metrics in payload");
            return None;
        }

        Some(environment_sensor::Entity {
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

impl DecodedRole for plant_sensor::Entity {
    type Message = plant_sensor::Message;

    const ACTOR: &'static str = PlantSensorHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.plant.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.plant(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;
        let declared = &device.profile.plant;

        let reported = reading.plant.as_ref()?;

        let readings: std::collections::BTreeMap<String, f64> = reported
            .iter()
            .filter(|(metric, _)| {
                let known = declared.contains(metric);

                if !known {
                    tracing::warn!(
                        "{} model {} reported undeclared plant metric {metric} for {address}",
                        device.profile.source(),
                        device.profile.slug
                    );
                }

                known
            })
            .map(|(metric, value)| (metric.clone(), *value))
            .collect();

        if readings.is_empty() {
            tracing::info!("skipping plant reading for {address}: no declared metrics");
            return None;
        }

        Some(plant_sensor::Entity {
            address: address.clone(),
            readings,
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        plant_sensor::Message::NewEvent(plant_sensor::NewEvent {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
            entity: self,
        })
    }
}

impl DecodedRole for light::Entity {
    type Message = light::LightHandlerMessage;

    const ACTOR: &'static str = LightHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.light.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.light(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let fields = reading.light.as_ref()?;

        let Some(state) = fields.state.clone() else {
            tracing::info!("skipping light reading for {address}: no state in payload");
            return None;
        };

        Some(light::Entity {
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

impl DecodedRole for smart_switch::Entity {
    type Message = smart_switch::Message;

    const ACTOR: &'static str = SmartSwitchHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.smart_switch.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.smart_switch(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let fields = reading.smart_switch.as_ref()?;

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

impl DecodedRole for presence_sensor::Entity {
    type Message = presence_sensor::Message;

    const ACTOR: &'static str = PresenceSensorHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.presence.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.presence(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let fields = reading.presence.as_ref()?;

        let Some(presence) = fields.presence else {
            tracing::info!("skipping presence reading for {address}: no presence value");
            return None;
        };

        Some(presence_sensor::Entity {
            address: address.clone(),
            sensor: fields.sensor.clone(),
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

impl DecodedRole for control_switch::Entity {
    type Message = control_switch::ControlSwitchMessage;

    const ACTOR: &'static str = ControlSwitchHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.control_switch.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.control_switch(address)
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let action = reading
            .control_switch
            .as_ref()?
            .action
            .clone()
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

impl DecodedRole for robot_vacuum::RobotVacuumReading {
    type Message = robot_vacuum::Message;

    const ACTOR: &'static str = RobotVacuumHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.robot_vacuum.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.robot_vacuum(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let fields = reading.robot_vacuum.as_ref()?;

        Some(robot_vacuum::RobotVacuumReading {
            device_id: device.id.clone(),
            protocol: device.profile.protocol,
            status: fields.status.clone(),
            room: fields.room.clone(),
            battery: fields.battery,
            fan_speed: fields.fan_speed.clone(),
            clean_area: fields.clean_area,
            clean_count: fields.clean_count,
            attributes: fields.attributes.clone(),
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        robot_vacuum::Message::NewEvent(robot_vacuum::NewEvent {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
            reading: self,
        })
    }
}

impl DecodedRole for media_player::MediaPlayerReading {
    type Message = media_player::Message;

    const ACTOR: &'static str = MediaPlayerHandler::NAME;

    fn present(reading: &DeviceReading) -> bool {
        reading.media_player.is_some()
    }

    fn declared(devices: &DeviceRegistry, address: &str) -> bool {
        devices.media_player(address).is_some()
    }

    fn extract(
        device: &DecodedDevice,
        _friendly_name: &str,
        reading: &DeviceReading,
    ) -> Option<Self> {
        let address = &device.address;

        let fields = reading.media_player.as_ref()?;

        let Some(state) = fields.state.clone() else {
            tracing::info!("skipping media player reading for {address}: no state");
            return None;
        };

        Some(media_player::MediaPlayerReading {
            address: address.clone(),
            state,
            attributes: fields
                .attributes
                .clone()
                .unwrap_or_else(|| serde_json::Value::Object(Default::default())),
        })
    }

    fn into_message(self, event_id: Uuid) -> Self::Message {
        media_player::Message::HomeAssistant(media_player::Update {
            event_id,
            traceparent: crate::tracing_context::inject_current(),
            reading: self,
        })
    }
}

pub fn run<R: DecodedRole>(
    event_id: Uuid,
    devices: &DeviceRegistry,
    device: &DecodedDevice,
    friendly_name: &str,
    reading: &DeviceReading,
) {
    if !R::present(reading) || !R::declared(devices, &device.address) {
        return;
    }

    let Some(entity) = R::extract(device, friendly_name, reading) else {
        return;
    };

    if let Err(e) = rpc::cast_factory(R::ACTOR, entity.into_message(event_id)) {
        tracing::error!(
            "failed to dispatch {} event to {}: {e}",
            device.profile.source(),
            R::ACTOR
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        decoding::load_models,
        device_metric::MetricValue,
        device_registry::Transport,
        integrations::esphome::EsphomeDomain,
        lua::LuaDecoder,
        settings::{LuaSettings, Metric},
    };
    use serde_json::{Map, Value};
    use std::collections::BTreeMap;

    struct TestDevice {
        device: DecodedDevice,
        decoder: LuaDecoder,
    }

    impl std::ops::Deref for TestDevice {
        type Target = DecodedDevice;

        fn deref(&self) -> &DecodedDevice {
            &self.device
        }
    }

    fn test_device(kind: Transport, slug: &str, source: &str, address: &str) -> TestDevice {
        let sources = BTreeMap::from([(slug.to_owned(), source.to_owned())]);
        let library =
            crate::lua::sources::load_directory(std::path::Path::new("./config/lua/model_lib"))
                .expect("model library");
        let models = load_models(
            kind,
            &sources,
            &library,
            &LuaSettings::default(),
            &crate::settings::MqttProtocols::committed(),
        )
        .expect("models");
        let decoder = LuaDecoder::load(
            &kind.to_string(),
            &sources,
            &library,
            &LuaSettings::default(),
        )
        .expect("decoder");

        TestDevice {
            device: DecodedDevice {
                id: "test-device".to_owned(),
                address: address.to_owned(),
                profile: models[slug].clone(),
            },
            decoder,
        }
    }

    fn device(slug: &str, source: &str) -> TestDevice {
        test_device(Transport::Mqtt, slug, source, "0xabc")
    }

    fn reading(device: &TestDevice, json: &str) -> DeviceReading {
        let payload: Map<String, Value> = serde_json::from_str(json).expect("payload");
        let input = serde_json::json!({
            "topic": "report",
            "vars": { "name": "test" },
            "payload": payload,
        });

        device
            .profile
            .decode(&device.decoder, &input)
            .expect("decode")
    }

    fn metrics(reading: DeviceReading) -> Vec<(String, MetricValue)> {
        reading
            .metrics
            .into_iter()
            .map(|(name, value)| (name, value.into()))
            .collect()
    }

    const AQARA_DOOR: &str = include_str!("../../config/lua/mqtt/aqara_mccgq12lm.lua");
    const LUMI_ENVIRONMENT: &str = include_str!("../../config/lua/mqtt/lumi_wsdcgq11lm.lua");
    const AQARA_FP1E: &str = include_str!("../../config/lua/mqtt/aqara_fp1e.lua");
    const TS011F_PLUG: &str = include_str!("../../config/lua/mqtt/ts011f_plug.lua");
    const AQARA_SWITCH: &str = include_str!("../../config/lua/mqtt/aqara_wxkg11lm.lua");
    const AQARA_T1: &str = include_str!("../../config/lua/mqtt/aqara_t1.lua");

    #[test]
    fn extracts_an_aqara_door_payload() {
        let device = device("aqara_mccgq12lm", AQARA_DOOR);
        let reading = reading(
            &device,
            r#"{"contact":false,"battery":97,"device_temperature":21,"voltage":3005,"linkquality":72}"#,
        );

        let Some(door_sensor::Entity::Decoded {
            contact,
            battery: level,
            friendly_name,
            ..
        }) = <door_sensor::Entity as DecodedRole>::extract(&device, "front-door", &reading)
        else {
            panic!("expected a door reading");
        };

        assert!(!contact);
        assert_eq!(level, Some(97));
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
            <door_sensor::Entity as DecodedRole>::extract(
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

        let Some(environment_sensor::Entity {
            readings,
            battery: level,
            ..
        }) = <environment_sensor::Entity as DecodedRole>::extract(&device, "outdoor", &reading)
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

        let Some(presence_sensor::Entity { presence, .. }) =
            <presence_sensor::Entity as DecodedRole>::extract(&device, "closet-presence", &reading)
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
        }) = <smart_switch::Entity as DecodedRole>::extract(
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
            <smart_switch::Entity as DecodedRole>::extract(
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

        let Some(light::Entity { attributes, .. }) = <light::Entity as DecodedRole>::extract(
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
            <control_switch::Entity as DecodedRole>::extract(
                &device,
                "small-switch",
                &reading(&device, r#"{"action":"","battery":91}"#)
            )
            .is_none(),
            "an empty action is not an event"
        );

        let Some(control_switch::Entity::Zigbee { action, .. }) =
            <control_switch::Entity as DecodedRole>::extract(
                &device,
                "small-switch",
                &reading(&device, r#"{"action":"single","battery":91}"#),
            )
        else {
            panic!("expected a control switch reading");
        };

        assert_eq!(action, "single");
    }

    const APOLLO_MTR_1: &str = include_str!("../../config/lua/mqtt/apollo_mtr_1.lua");

    fn esphome(
        device: &TestDevice,
        domain: EsphomeDomain,
        object_id: &str,
        payload: &[u8],
    ) -> DeviceReading {
        let input = serde_json::json!({
            "topic": "state",
            "vars": {
                "address": device.address,
                "domain": domain,
                "object_id": object_id,
            },
            "payload": domain.parse(payload).expect("payload"),
        });

        device
            .profile
            .decode(&device.decoder, &input)
            .expect("decode")
    }

    #[test]
    fn an_esphome_node_decodes_each_entity_into_its_role() {
        let device = test_device(
            Transport::Mqtt,
            "apollo_mtr_1",
            APOLLO_MTR_1,
            "apollo-mtr-1-livingroom",
        );

        let temperature = esphome(
            &device,
            EsphomeDomain::Sensor,
            "dps310_temperature",
            b"21.5",
        );

        let Some(environment_sensor::Entity { readings, .. }) =
            <environment_sensor::Entity as DecodedRole>::extract(
                &device,
                "Living Room",
                &temperature,
            )
        else {
            panic!("expected an environment reading");
        };

        assert_eq!(readings, [(Metric::Temperature, 21.5)]);

        let motion = esphome(
            &device,
            EsphomeDomain::BinarySensor,
            "ld2450_still_target",
            b"ON",
        );

        let Some(presence_sensor::Entity {
            presence, sensor, ..
        }) = <presence_sensor::Entity as DecodedRole>::extract(&device, "Living Room", &motion)
        else {
            panic!("expected a presence reading");
        };

        assert!(presence);
        assert_eq!(sensor.as_deref(), Some("ld2450_still_target"));

        let light = esphome(
            &device,
            EsphomeDomain::Light,
            "rgb_light",
            br#"{"state":"ON","brightness":255,"color":{"r":255,"g":136,"b":0}}"#,
        );

        let Some(light::Entity { attributes, .. }) =
            <light::Entity as DecodedRole>::extract(&device, "Living Room", &light)
        else {
            panic!("expected a light reading");
        };

        assert_eq!(attributes.state.as_deref(), Some("ON"));
        assert_eq!(attributes.brightness, Some(254));
        assert_eq!(attributes.colour.as_deref(), Some("#ff8800"));
    }

    const VALETUDO: &str = include_str!("../../config/lua/mqtt/valetudo.lua");

    fn valetudo(topic: &str, payload: Value) -> robot_vacuum::RobotVacuumReading {
        let device = test_device(Transport::Mqtt, "valetudo", VALETUDO, "rockrobo");
        let input = serde_json::json!({
            "topic": topic,
            "vars": { "address": "rockrobo" },
            "payload": payload,
        });

        let reading = device
            .profile
            .decode(&device.decoder, &input)
            .expect("decode");

        <robot_vacuum::RobotVacuumReading as DecodedRole>::extract(&device, "Vacuum", &reading)
            .expect("a robot vacuum reading")
    }

    #[test]
    fn valetudo_state_and_attributes_map_to_the_robot_vacuum_role() {
        let state = valetudo(
            "state",
            serde_json::json!({ "state": "docked", "battery_level": 87, "fan_speed": "max" }),
        );
        assert_eq!(
            state.protocol,
            Some(crate::integrations::mqtt::MqttProtocol::Valetudo)
        );
        assert_eq!(state.status.as_deref(), Some("docked"));
        assert_eq!(state.battery, Some(87));
        assert_eq!(state.fan_speed.as_deref(), Some("max"));
        assert!(state.attributes.is_none());

        let attributes = valetudo(
            "attributes",
            serde_json::json!({ "currentCleanArea": "12.5", "cleanCount": 4 }),
        );
        assert_eq!(attributes.clean_area, Some(12.5));
        assert_eq!(attributes.clean_count, Some(4));
        assert!(attributes.attributes.is_some());
    }

    const ROBOROCK: &str = include_str!("../../config/lua/home_assistant/roborock.lua");
    const MEDIA_PLAYER: &str = include_str!("../../config/lua/home_assistant/media_player.lua");

    fn entity_device(slug: &str, source: &str, address: &str) -> TestDevice {
        test_device(Transport::HomeAssistant, slug, source, address)
    }

    fn entity(device: &TestDevice, entity_id: &str, state: &str) -> DeviceReading {
        let entity = serde_json::json!({
            "entity_id": entity_id,
            "state": state,
            "attributes": { "friendly_name": "Robot" },
        });

        device
            .profile
            .decode(&device.decoder, &entity)
            .expect("decode")
    }

    fn roborock(reading: &DeviceReading) -> Option<robot_vacuum::RobotVacuumReading> {
        let device = entity_device("roborock", ROBOROCK, "vacuum.robot");

        <robot_vacuum::RobotVacuumReading as DecodedRole>::extract(&device, "Robot", reading)
    }

    #[test]
    fn roborock_status_room_and_battery_map_to_the_robot_vacuum_role() {
        let device = entity_device("roborock", ROBOROCK, "vacuum.robot");

        let status = roborock(&entity(&device, "sensor.robot_status", "charging")).expect("status");
        assert_eq!(status.device_id, "test-device");
        assert_eq!(status.status.as_deref(), Some("charging"));
        assert_eq!(status.room, None);
        assert_eq!(status.battery, None);

        let room =
            roborock(&entity(&device, "sensor.robot_current_room", "Dining room")).expect("room");
        assert_eq!(room.room.as_deref(), Some("Dining room"));

        let battery = roborock(&entity(&device, "sensor.robot_battery", "100")).expect("battery");
        assert_eq!(battery.battery, Some(100));
    }

    #[test]
    fn roborock_extra_entities_land_as_typed_metrics() {
        let device = entity_device("roborock", ROBOROCK, "vacuum.robot");

        let shortage = entity(&device, "binary_sensor.robot_water_shortage", "on");
        assert!(
            roborock(&shortage).is_none(),
            "a metric is not a status update"
        );
        assert_eq!(
            metrics(shortage),
            vec![(
                "robot_water_shortage".to_owned(),
                MetricValue::Text("true".to_owned())
            )]
        );

        assert_eq!(
            metrics(entity(&device, "sensor.robot_filter_time_left", "110.09")),
            vec![(
                "robot_filter_time_left".to_owned(),
                MetricValue::Numeric(110.09)
            )]
        );
        assert_eq!(
            metrics(entity(&device, "sensor.robot_vacuum_error", "none")),
            vec![(
                "robot_vacuum_error".to_owned(),
                MetricValue::Text("none".to_owned())
            )]
        );
        assert_eq!(
            metrics(entity(&device, "vacuum.robot", "docked")),
            vec![("robot".to_owned(), MetricValue::Text("docked".to_owned()))]
        );
    }

    #[test]
    fn an_unavailable_roborock_entity_reports_nothing() {
        let device = entity_device("roborock", ROBOROCK, "vacuum.robot");
        let reading = entity(&device, "sensor.robot_status", "unavailable");

        assert!(roborock(&reading).is_none());
        assert!(reading.metrics.is_empty());
    }

    #[test]
    fn a_media_player_passes_state_and_attributes_through() {
        let device = entity_device("media_player", MEDIA_PLAYER, "media_player.living_room_tv");
        let payload = serde_json::json!({
            "entity_id": "media_player.living_room_tv",
            "state": "playing",
            "attributes": { "media_title": "Severance", "app_name": "Apple TV", "volume_level": 0.3 },
        });

        let reading = device
            .profile
            .decode(&device.decoder, &payload)
            .expect("decode");

        let Some(media_player::MediaPlayerReading {
            address,
            state,
            attributes,
        }) = <media_player::MediaPlayerReading as DecodedRole>::extract(&device, "TV", &reading)
        else {
            panic!("expected a media player reading");
        };

        assert_eq!(address, "media_player.living_room_tv");
        assert_eq!(state, "playing");
        assert_eq!(attributes["media_title"], "Severance");
        assert_eq!(attributes["app_name"], "Apple TV");
    }
}
