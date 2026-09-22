use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use crate::device_registry::{Capability, Transport};
use crate::integrations::mqtt::MqttProtocol;
use crate::lua::{LuaDecoder, LuaError};
use crate::settings::{LuaSettings, Metric, MqttProtocols};

use super::esphome_entities::EsphomeEntities;
use super::home_assistant_entities::HomeAssistantEntities;
use super::model_commands::ModelCommands;
use super::model_entities::ModelEntities;
use super::model_profile::ModelProfile;
use super::model_ranges::ModelRanges;
use super::role_name::DeviceRoleName;

pub type Models = HashMap<String, Arc<ModelProfile>>;

pub fn load_models(
    kind: Transport,
    sources: &BTreeMap<String, String>,
    library: &BTreeMap<String, String>,
    settings: &LuaSettings,
    protocols: &MqttProtocols,
) -> Result<Models, String> {
    let decoder = LuaDecoder::load(&kind.to_string(), sources, library, settings)
        .map_err(|error| format!("{kind} models: {error}"))?;

    let mut profiles = HashMap::new();

    for slug in decoder.modules() {
        let profile = resolve_profile(kind, &decoder, slug, protocols)?;

        profiles.insert(slug.to_owned(), Arc::new(profile));
    }

    Ok(profiles)
}

fn resolve_profile(
    kind: Transport,
    decoder: &LuaDecoder,
    slug: &str,
    protocols: &MqttProtocols,
) -> Result<ModelProfile, String> {
    let error = |error: LuaError| format!("{kind} model {slug}: {error}");

    if !decoder.has_function(slug, "decode").map_err(error)? {
        return Err(format!(
            "{kind} model {slug}: must define a `decode` function"
        ));
    }

    let Some(declared) = decoder
        .field::<Vec<DeviceRoleName>>(slug, "roles")
        .map_err(error)?
    else {
        return Err(format!("{kind} model {slug}: must declare `roles`"));
    };

    let mut roles = BTreeSet::new();

    for role in declared {
        if !roles.insert(role) {
            return Err(format!(
                "{kind} model {slug}: `roles` declares `{role}` twice"
            ));
        }
    }

    if roles.is_empty() {
        return Err(format!("{kind} model {slug}: `roles` is empty"));
    }

    let protocol = decoder
        .field::<MqttProtocol>(slug, "protocol")
        .map_err(error)?;

    let protocol = match (kind, protocol) {
        (Transport::Mqtt, Some(protocol)) => Some(protocol),
        (Transport::Mqtt, None) => {
            return Err(format!("{kind} model {slug}: must declare a `protocol`"));
        }
        (_, Some(_)) => {
            return Err(format!(
                "{kind} model {slug}: only mqtt models declare a `protocol`"
            ));
        }
        (_, None) => None,
    };

    let carrier = protocol.map_or_else(|| kind.to_string(), |protocol| protocol.to_string());

    let supported = |role: DeviceRoleName| match protocol {
        Some(protocol) => protocols.get(protocol).supports(role),
        None => kind.supports(role),
    };

    if let Some(role) = roles.iter().find(|role| !supported(**role)) {
        return Err(format!(
            "{kind} model {slug}: the {carrier} protocol can't carry the `{role}` role"
        ));
    }

    let capabilities = decoder
        .field::<Vec<Capability>>(slug, "capabilities")
        .map_err(error)?
        .unwrap_or_default();

    let environment: Vec<Metric> = capabilities
        .iter()
        .filter_map(|capability| capability.metric())
        .collect();

    let reports_environment = roles.contains(&DeviceRoleName::Environment);

    if reports_environment && !environment.contains(&Metric::Temperature) {
        return Err(format!(
            "{kind} model {slug}: `capabilities` must list `temperature` for the `environment` role"
        ));
    }

    if !reports_environment && !environment.is_empty() {
        return Err(format!(
            "{kind} model {slug}: lists environment capabilities without declaring the `environment` role"
        ));
    }

    let plant = decoder
        .field::<Vec<String>>(slug, "plant")
        .map_err(error)?
        .unwrap_or_default();

    if roles.contains(&DeviceRoleName::Plant) == plant.is_empty() {
        return Err(format!(
            "{kind} model {slug}: `plant` metrics are required with the `plant` role, and only with it"
        ));
    }

    let watchdog = decoder
        .field::<String>(slug, "watchdog")
        .map_err(error)?
        .map(|timeout| {
            crate::timedelta_format::parse_datetime_str_with_ms(&timeout)
                .map_err(|e| format!("{kind} model {slug}: invalid `watchdog` `{timeout}`: {e}"))
        })
        .transpose()?;

    let entities = resolve_entities(kind, protocol, protocols, decoder, slug, &roles)?;
    let commands = resolve_commands(kind, decoder, slug, &roles)?;
    let ranges = resolve_ranges(kind, decoder, slug, &capabilities)?;

    Ok(ModelProfile {
        transport: kind,
        protocol,
        slug: slug.to_owned(),
        roles,
        capabilities,
        environment,
        plant,
        entities,
        commands,
        ranges,
        watchdog,
    })
}

fn resolve_ranges(
    kind: Transport,
    decoder: &LuaDecoder,
    slug: &str,
    capabilities: &[Capability],
) -> Result<ModelRanges, String> {
    let ranges = decoder
        .field::<ModelRanges>(slug, "ranges")
        .map_err(|error| format!("{kind} model {slug}: {error}"))?
        .unwrap_or_default();

    let colour_temp = capabilities.contains(&Capability::ColourTemp);

    if colour_temp != ranges.colour_temp.is_some() {
        return Err(format!(
            "{kind} model {slug}: `ranges.colour_temp` is required with the `colour_temp` capability, and only with it"
        ));
    }

    if let Some(range) = ranges.colour_temp
        && range.min >= range.max
    {
        return Err(format!(
            "{kind} model {slug}: `ranges.colour_temp` min {} must be below max {}",
            range.min, range.max
        ));
    }

    Ok(ranges)
}

fn resolve_commands(
    kind: Transport,
    decoder: &LuaDecoder,
    slug: &str,
    roles: &BTreeSet<DeviceRoleName>,
) -> Result<ModelCommands, String> {
    let commands = decoder
        .field::<ModelCommands>(slug, "commands")
        .map_err(|error| format!("{kind} model {slug}: {error}"))?
        .unwrap_or_default();

    let vacuum = roles.contains(&DeviceRoleName::RobotVacuum);

    if vacuum != commands.robot_vacuum.is_some() {
        return Err(format!(
            "{kind} model {slug}: `commands.robot_vacuum` is required with the `robot_vacuum` role, and only with it"
        ));
    }

    for role in [DeviceRoleName::Light, DeviceRoleName::SmartSwitch] {
        let name = role.to_string();
        let encodes = decoder
            .has_function_at(slug, &["encode", &name])
            .map_err(|error| format!("{kind} model {slug}: {error}"))?;

        if roles.contains(&role) != encodes {
            return Err(format!(
                "{kind} model {slug}: `encode.{name}` is required with the `{name}` role, and only with it"
            ));
        }
    }

    Ok(commands)
}

fn resolve_entities(
    kind: Transport,
    protocol: Option<MqttProtocol>,
    protocols: &MqttProtocols,
    decoder: &LuaDecoder,
    slug: &str,
    roles: &BTreeSet<DeviceRoleName>,
) -> Result<ModelEntities, String> {
    let error = |error: LuaError| format!("{kind} model {slug}: {error}");

    match (kind, protocol) {
        (Transport::Mqtt, Some(protocol)) if !protocols.get(protocol).takes_entities() => {
            let declared = decoder
                .field::<serde_json::Value>(slug, "entities")
                .map_err(error)?;

            if declared.is_some() {
                return Err(format!(
                    "{kind} model {slug}: {protocol} devices report on fixed topics, so `entities` is not allowed"
                ));
            }

            Ok(ModelEntities::Payload)
        }
        (Transport::Mqtt, _) => {
            let Some(entities) = decoder
                .field::<EsphomeEntities>(slug, "entities")
                .map_err(error)?
            else {
                return Err(format!("{kind} model {slug}: must declare `entities`"));
            };

            if entities.is_empty() {
                return Err(format!("{kind} model {slug}: `entities` is empty"));
            }

            let lights = roles.contains(&DeviceRoleName::Light);

            if lights && entities.light().is_none() {
                return Err(format!(
                    "{kind} model {slug}: the `light` role needs exactly one `light` entity"
                ));
            }

            if !lights && !entities.light.is_empty() {
                return Err(format!(
                    "{kind} model {slug}: lists `light` entities without declaring the `light` role"
                ));
            }

            Ok(ModelEntities::Esphome(entities))
        }
        (Transport::HomeAssistant, _) => {
            let entities = decoder
                .field::<HomeAssistantEntities>(slug, "entities")
                .map_err(error)?
                .unwrap_or_default();

            Ok(ModelEntities::HomeAssistant(entities))
        }
        (Transport::EinkDisplayFirmware | Transport::Trmnl, _) => Err(format!(
            "{kind} model {slug}: the {kind} transport does not take models"
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Map, Value};

    use super::*;
    use crate::decoding::reading::{DeviceReading, ReadingMetric};
    use crate::device_metric::MetricValue;

    fn load(source: &str) -> Result<Models, String> {
        load_protocol("zigbee", source)
    }

    fn load_protocol(protocol: &str, source: &str) -> Result<Models, String> {
        let source = source.replacen(
            "return {",
            &format!("return {{ protocol = \"{protocol}\","),
            1,
        );

        load_as(Transport::Mqtt, &source)
    }

    fn load_as(transport: Transport, source: &str) -> Result<Models, String> {
        let sources = BTreeMap::from([("test_model".to_owned(), source.to_owned())]);

        load_models(
            transport,
            &sources,
            &BTreeMap::new(),
            &LuaSettings::default(),
            &MqttProtocols::committed(),
        )
    }

    fn profile(source: &str) -> Arc<ModelProfile> {
        load(source).expect("model")["test_model"].clone()
    }

    fn payload(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("payload json")
    }

    fn decode(source: &str, json: &str) -> Result<DeviceReading, LuaError> {
        let sources = BTreeMap::from([("test_model".to_owned(), source.to_owned())]);
        let decoder = LuaDecoder::load(
            "zigbee",
            &sources,
            &BTreeMap::new(),
            &LuaSettings::default(),
        )
        .expect("decoder");

        profile(source).decode(&decoder, &payload(json))
    }

    #[test]
    fn roles_and_environment_metrics_are_read_at_load() {
        let profile = profile(
            r#"return {
                roles = { "battery", "environment" },
                capabilities = { "temperature", "pm25" },
                decode = function(p) return {} end,
            }"#,
        );

        assert!(profile.roles.contains(&DeviceRoleName::Battery));
        assert!(profile.roles.contains(&DeviceRoleName::Environment));
        assert_eq!(
            profile.capabilities,
            [Capability::Temperature, Capability::Pm25]
        );
        assert_eq!(profile.environment, [Metric::Temperature, Metric::Pm25]);
    }

    #[test]
    fn a_role_the_transport_cannot_carry_is_rejected() {
        let error =
            load(r#"return { roles = { "media_player" }, decode = function(p) return {} end }"#)
                .expect_err("zigbee media player");

        assert!(
            error.contains("can't carry the `media_player` role"),
            "{error}"
        );
    }

    #[test]
    fn a_zigbee_model_cannot_list_entities() {
        let error = load(
            r#"return { roles = { "door" }, entities = { "x" }, decode = function(p) return {} end }"#,
        )
        .expect_err("zigbee entities");

        assert!(error.contains("`entities` is not allowed"), "{error}");
    }

    #[test]
    fn an_esphome_model_needs_entities_and_one_light_for_the_light_role() {
        let missing = load_protocol(
            "esphome",
            r#"return { roles = { "presence" }, decode = function(e) return {} end }"#,
        )
        .expect_err("no entities");
        assert!(missing.contains("must declare `entities`"), "{missing}");

        let lights = load_protocol(
            "esphome",
            r#"return {
                roles = { "light" },
                entities = { light = { "a", "b" } },
                encode = { light = function(i) return nil end },
                decode = function(e) return {} end,
            }"#,
        )
        .expect_err("two lights");
        assert!(lights.contains("exactly one `light` entity"), "{lights}");

        let models = load_protocol(
            "esphome",
            r#"return {
                roles = { "light", "presence" },
                entities = { light = { "rgb" }, binary_sensor = { "motion" } },
                encode = { light = function(i) return nil end },
                decode = function(e) return {} end,
            }"#,
        )
        .expect("esphome model");

        let ModelEntities::Esphome(entities) = &models["test_model"].entities else {
            panic!("expected esphome entities");
        };

        assert_eq!(entities.light(), Some("rgb"));
        assert_eq!(entities.binary_sensor, ["motion"]);
    }

    #[test]
    fn the_plant_role_and_plant_metrics_come_together() {
        let missing = load_protocol(
            "esphome",
            r#"return {
                roles = { "plant" },
                entities = { sensor = { "soil_moisture" } },
                decode = function(e) return {} end,
            }"#,
        )
        .expect_err("plant without metrics");

        assert!(
            missing.contains("`plant` metrics are required"),
            "{missing}"
        );
    }

    #[test]
    fn a_model_without_roles_is_rejected() {
        let error = load("return { decode = function(p) return {} end }").expect_err("no roles");

        assert!(error.contains("must declare `roles`"), "{error}");
    }

    #[test]
    fn a_model_without_decode_is_rejected() {
        let error = load(r#"return { roles = { "door" } }"#).expect_err("no decode");

        assert!(error.contains("must define a `decode` function"), "{error}");
    }

    #[test]
    fn an_unknown_role_is_rejected() {
        let error = load(r#"return { roles = { "toaster" }, decode = function(p) return {} end }"#)
            .expect_err("unknown role");

        assert!(error.contains("toaster"), "{error}");
    }

    #[test]
    fn a_repeated_role_is_rejected() {
        let error =
            load(r#"return { roles = { "door", "door" }, decode = function(p) return {} end }"#)
                .expect_err("repeated role");

        assert!(error.contains("declares `door` twice"), "{error}");
    }

    #[test]
    fn environment_requires_temperature_and_rejects_unknown_metrics() {
        let missing = load(
            r#"return { roles = { "environment" }, capabilities = { "humidity" }, decode = function(p) return {} end }"#,
        )
        .expect_err("no temperature");
        assert!(missing.contains("must list `temperature`"), "{missing}");

        let unknown = load(
            r#"return { roles = { "environment" }, capabilities = { "temperature", "wind_speed" }, decode = function(p) return {} end }"#,
        )
        .expect_err("bad metric");
        assert!(unknown.contains("wind_speed"), "{unknown}");
    }

    #[test]
    fn environment_metrics_without_the_role_are_rejected() {
        let error = load(
            r#"return { roles = { "door" }, capabilities = { "temperature" }, decode = function(p) return {} end }"#,
        )
        .expect_err("environment without role");

        assert!(
            error.contains("without declaring the `environment` role"),
            "{error}"
        );
    }

    #[test]
    fn decode_maps_payload_fields_into_a_typed_reading() {
        let reading = decode(
            r#"return {
                roles = { "battery", "door" },
                decode = function(p)
                    return {
                        battery = p.battery,
                        door = { contact = p.contact },
                        metrics = { voltage = p.voltage, open = not p.contact, mode = p.mode },
                    }
                end,
            }"#,
            r#"{"battery": 97, "contact": false, "voltage": 3005, "mode": "eco"}"#,
        )
        .expect("decode");

        assert_eq!(reading.battery, Some(97));
        assert_eq!(reading.door.and_then(|door| door.contact), Some(false));
        assert_eq!(
            MetricValue::from(reading.metrics["voltage"].clone()),
            MetricValue::Numeric(3005.0)
        );
        assert_eq!(reading.metrics["open"], ReadingMetric::Flag(true));
        assert_eq!(
            MetricValue::from(reading.metrics["mode"].clone()),
            MetricValue::Text("eco".to_owned())
        );
    }

    #[test]
    fn json_nulls_arrive_in_lua_as_nil() {
        let reading = decode(
            r#"return {
                roles = { "door" },
                decode = function(p) return { door = { contact = p.contact } } end,
            }"#,
            r#"{"contact": null}"#,
        )
        .expect("decode");

        assert_eq!(reading.door.and_then(|door| door.contact), None);
    }

    #[test]
    fn an_unknown_block_in_a_reading_is_an_error() {
        let error = decode(
            r#"return { roles = { "door" }, decode = function(p) return { garage = {} } end }"#,
            "{}",
        )
        .expect_err("unknown block");

        assert!(error.to_string().contains("garage"), "{error}");
    }

    #[test]
    fn an_mqtt_model_must_declare_its_protocol() {
        let error = load_as(
            Transport::Mqtt,
            r#"return { roles = { "door" }, decode = function(p) return {} end }"#,
        )
        .expect_err("no protocol");

        assert!(error.contains("must declare a `protocol`"), "{error}");
    }

    #[test]
    fn only_mqtt_models_declare_a_protocol() {
        let error = load_as(
            Transport::HomeAssistant,
            r#"return { protocol = "zigbee", roles = { "door" }, decode = function(p) return {} end }"#,
        )
        .expect_err("protocol on home assistant");

        assert!(error.contains("only mqtt models"), "{error}");
    }

    #[test]
    fn a_role_the_protocol_cannot_carry_is_rejected() {
        let error = load_protocol(
            "valetudo",
            r#"return { roles = { "door" }, decode = function(p) return {} end }"#,
        )
        .expect_err("valetudo door");

        assert!(error.contains("valetudo protocol can't carry"), "{error}");
    }

    #[test]
    fn robot_vacuum_commands_come_with_the_role() {
        let missing = load_protocol(
            "valetudo",
            r#"return { roles = { "robot_vacuum" }, decode = function(p) return {} end }"#,
        )
        .expect_err("vacuum without commands");
        assert!(
            missing.contains("`commands.robot_vacuum` is required"),
            "{missing}"
        );

        let stray = load_protocol(
            "zigbee",
            r#"return {
                roles = { "door" },
                commands = { robot_vacuum = { start = "a", stop = "b", dock = "c" } },
                decode = function(p) return {} end,
            }"#,
        )
        .expect_err("commands without vacuum");
        assert!(stray.contains("and only with it"), "{stray}");

        let models = load_protocol(
            "valetudo",
            r#"return {
                roles = { "robot_vacuum" },
                commands = { robot_vacuum = { start = "START", stop = "STOP", dock = "HOME" } },
                decode = function(p) return {} end,
            }"#,
        )
        .expect("valetudo model");

        let commands = models["test_model"]
            .commands
            .robot_vacuum
            .as_ref()
            .expect("commands");
        assert_eq!(commands.dock, "HOME");
    }

    fn library() -> BTreeMap<String, String> {
        crate::lua::sources::load_directory(std::path::Path::new("./config/lua/model_lib"))
            .expect("expected the committed model library to be readable")
    }

    #[test]
    fn every_committed_mqtt_model_loads() {
        let sources =
            crate::lua::sources::load_directory(std::path::Path::new("./config/lua/mqtt"))
                .expect("expected the committed mqtt models to be readable");

        let models = load_models(
            Transport::Mqtt,
            &sources,
            &library(),
            &LuaSettings::default(),
            &MqttProtocols::committed(),
        )
        .expect("expected every committed mqtt model to load");

        assert!(models.contains_key("apollo_mtr_1"));
        assert!(models.contains_key("valetudo"));
        assert_eq!(models.len(), sources.len());
    }

    #[test]
    fn every_committed_home_assistant_model_loads() {
        let sources = crate::lua::sources::load_directory(std::path::Path::new(
            "./config/lua/home_assistant",
        ))
        .expect("expected the committed home assistant models to be readable");

        let models = load_models(
            Transport::HomeAssistant,
            &sources,
            &library(),
            &LuaSettings::default(),
            &MqttProtocols::committed(),
        )
        .expect("expected every committed home assistant model to load");

        assert!(models.contains_key("roborock"));
        assert!(models.contains_key("media_player"));
        assert_eq!(models.len(), sources.len());
    }

    #[test]
    fn the_light_role_and_a_light_encoder_come_together() {
        let missing = load(r#"return { roles = { "light" }, decode = function(p) return {} end }"#)
            .expect_err("no encoder");
        assert!(missing.contains("`encode.light` is required"), "{missing}");

        let stray = load(
            r#"return {
                roles = { "door" },
                encode = { light = function(i) return nil end },
                decode = function(p) return {} end,
            }"#,
        )
        .expect_err("encoder without the role");
        assert!(stray.contains("`encode.light` is required"), "{stray}");
    }

    #[test]
    fn colour_temp_needs_a_range() {
        let missing = load(
            r#"return {
                roles = { "door" },
                capabilities = { "colour_temp" },
                decode = function(p) return {} end,
            }"#,
        )
        .expect_err("no range");
        assert!(
            missing.contains("`ranges.colour_temp` is required"),
            "{missing}"
        );

        let inverted = load(
            r#"return {
                roles = { "door" },
                capabilities = { "colour_temp" },
                ranges = { colour_temp = { min = 500, max = 153 } },
                decode = function(p) return {} end,
            }"#,
        )
        .expect_err("inverted range");
        assert!(inverted.contains("must be below max"), "{inverted}");

        let ranged = profile(
            r#"return {
                roles = { "door" },
                capabilities = { "colour_temp" },
                ranges = { colour_temp = { min = 250, max = 454 } },
                decode = function(p) return {} end,
            }"#,
        );
        let range = ranged.ranges.colour_temp.expect("range");
        assert_eq!(range.clamp(153), 250);
        assert_eq!(range.clamp(500), 454);
    }

    fn encode(slug: &str, command: serde_json::Value, on: bool) -> Option<Value> {
        encode_as(slug, DeviceRoleName::Light, command, on)
    }

    fn encode_as(
        slug: &str,
        role: DeviceRoleName,
        command: serde_json::Value,
        on: bool,
    ) -> Option<Value> {
        let sources =
            crate::lua::sources::load_directory(std::path::Path::new("./config/lua/mqtt"))
                .expect("committed mqtt models");
        let library = library();
        let models = load_models(
            Transport::Mqtt,
            &sources,
            &library,
            &LuaSettings::default(),
            &MqttProtocols::committed(),
        )
        .expect("committed models");
        let decoder =
            LuaDecoder::load("mqtt", &sources, &library, &LuaSettings::default()).expect("decoder");

        let input = serde_json::json!({
            "command": command,
            "current": { "on": on },
        });

        models[slug].encode(&decoder, role, &input).expect("encode")
    }

    #[test]
    fn the_zigbee_switch_encoder_only_switches() {
        use serde_json::json;

        let switch = |command| encode_as("ts011f_plug", DeviceRoleName::SmartSwitch, command, true);

        assert_eq!(
            switch(json!({ "type": "set", "on": false })),
            Some(json!({ "state": "OFF" }))
        );
        assert_eq!(
            switch(json!({ "type": "toggle" })),
            Some(json!({ "state": "TOGGLE" }))
        );
        assert_eq!(switch(json!({ "type": "set", "brightness": 100 })), None);
    }

    #[test]
    fn the_zigbee_encoder_speaks_zigbee2mqtt() {
        use serde_json::json;

        assert_eq!(
            encode(
                "aqara_t1",
                json!({ "type": "set", "on": true, "brightness": 120, "colour": "#ff8000" }),
                false
            ),
            Some(json!({ "state": "ON", "brightness": 120, "color": { "hex": "#ff8000" } }))
        );
        assert_eq!(
            encode("aqara_t1", json!({ "type": "toggle" }), true),
            Some(json!({ "state": "TOGGLE" }))
        );
        assert_eq!(
            encode(
                "aqara_t1",
                json!({ "type": "brightness_move", "value": -40, "on_off": true }),
                false
            ),
            Some(json!({ "brightness_move_onoff": -40 }))
        );
        assert_eq!(
            encode(
                "aqara_t1",
                json!({ "type": "colour_temp_move", "value": 0 }),
                false
            ),
            Some(json!({ "color_temp_move": "stop" }))
        );
    }

    #[test]
    fn the_esphome_encoder_rescales_and_resolves_toggles() {
        use serde_json::json;

        assert_eq!(
            encode(
                "apollo_mtr_1",
                json!({ "type": "set", "brightness": 254, "colour": "#ff8000" }),
                false
            ),
            Some(json!({ "brightness": 255, "color": { "r": 255, "g": 128, "b": 0 } }))
        );
        assert_eq!(
            encode(
                "apollo_mtr_1",
                json!({ "type": "set", "brightness": 0 }),
                false
            ),
            Some(json!({ "brightness": 0 }))
        );
        assert_eq!(
            encode("apollo_mtr_1", json!({ "type": "toggle" }), true),
            Some(json!({ "state": "OFF" }))
        );

        for command in [
            json!({ "type": "set", "colour_temp": 300 }),
            json!({ "type": "brightness_move", "value": 40, "on_off": false }),
            json!({ "type": "colour_temp_move", "value": 0 }),
        ] {
            assert_eq!(
                encode("apollo_mtr_1", command.clone(), false),
                None,
                "{command}"
            );
        }
    }
}
