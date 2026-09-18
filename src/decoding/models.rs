use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use crate::lua::{LuaDecoder, LuaError};
use crate::settings::{LuaSettings, Metric};

use super::model_profile::ModelProfile;
use super::role_name::DeviceRoleName;

pub type Models = HashMap<String, Arc<ModelProfile>>;

pub fn load_models(
    kind: &'static str,
    sources: &BTreeMap<String, String>,
    settings: &LuaSettings,
) -> Result<Models, String> {
    let decoder = LuaDecoder::load(kind, sources, settings)
        .map_err(|error| format!("{kind} models: {error}"))?;
    let decoder = Arc::new(decoder);

    let mut profiles = HashMap::new();

    for slug in decoder.modules() {
        let profile = resolve_profile(kind, &decoder, slug)?;

        profiles.insert(slug.to_owned(), Arc::new(profile));
    }

    Ok(profiles)
}

fn resolve_profile(
    kind: &'static str,
    decoder: &Arc<LuaDecoder>,
    slug: &str,
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

    let environment = decoder
        .field::<Vec<Metric>>(slug, "environment")
        .map_err(error)?
        .unwrap_or_default();

    let reports_environment = roles.contains(&DeviceRoleName::Environment);

    if reports_environment && !environment.contains(&Metric::Temperature) {
        return Err(format!(
            "{kind} model {slug}: `environment` must list `temperature`"
        ));
    }

    if !reports_environment && !environment.is_empty() {
        return Err(format!(
            "{kind} model {slug}: lists `environment` metrics without declaring the `environment` role"
        ));
    }

    Ok(ModelProfile::new(
        kind,
        slug.to_owned(),
        roles,
        environment,
        decoder.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Map, Value};

    use super::*;
    use crate::decoding::reading::ReadingMetric;
    use crate::device_metric::MetricValue;

    fn load(source: &str) -> Result<Models, String> {
        let sources = BTreeMap::from([("test_model".to_owned(), source.to_owned())]);

        load_models("zigbee", &sources, &LuaSettings::default())
    }

    fn profile(source: &str) -> Arc<ModelProfile> {
        load(source).expect("model")["test_model"].clone()
    }

    fn payload(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("payload json")
    }

    #[test]
    fn roles_and_environment_metrics_are_read_at_load() {
        let profile = profile(
            r#"return {
                roles = { "battery", "environment" },
                environment = { "temperature", "pm25" },
                decode = function(p) return {} end,
            }"#,
        );

        assert!(profile.roles.contains(&DeviceRoleName::Battery));
        assert!(profile.roles.contains(&DeviceRoleName::Environment));
        assert_eq!(profile.environment, [Metric::Temperature, Metric::Pm25]);
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
            r#"return { roles = { "environment" }, environment = { "humidity" }, decode = function(p) return {} end }"#,
        )
        .expect_err("no temperature");
        assert!(missing.contains("must list `temperature`"), "{missing}");

        let unknown = load(
            r#"return { roles = { "environment" }, environment = { "temperature", "wind_speed" }, decode = function(p) return {} end }"#,
        )
        .expect_err("bad metric");
        assert!(unknown.contains("wind_speed"), "{unknown}");
    }

    #[test]
    fn environment_metrics_without_the_role_are_rejected() {
        let error = load(
            r#"return { roles = { "door" }, environment = { "temperature" }, decode = function(p) return {} end }"#,
        )
        .expect_err("environment without role");

        assert!(
            error.contains("without declaring the `environment` role"),
            "{error}"
        );
    }

    #[test]
    fn decode_maps_payload_fields_into_a_typed_reading() {
        let profile = profile(
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
        );

        let reading = profile
            .decode(&payload(
                r#"{"battery": 97, "contact": false, "voltage": 3005, "mode": "eco"}"#,
            ))
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
        let profile = profile(
            r#"return {
                roles = { "door" },
                decode = function(p) return { door = { contact = p.contact } } end,
            }"#,
        );

        let reading = profile
            .decode(&payload(r#"{"contact": null}"#))
            .expect("decode");

        assert_eq!(reading.door.and_then(|door| door.contact), None);
    }

    #[test]
    fn an_unknown_block_in_a_reading_is_an_error() {
        let profile = profile(
            r#"return { roles = { "door" }, decode = function(p) return { garage = {} } end }"#,
        );

        let error = profile.decode(&payload("{}")).expect_err("unknown block");

        assert!(error.to_string().contains("garage"), "{error}");
    }

    #[test]
    fn every_committed_model_loads() {
        let sources =
            crate::lua::sources::load_directory(std::path::Path::new("./config/lua/zigbee"))
                .expect("expected the committed zigbee models to be readable");

        let models = load_models("zigbee", &sources, &LuaSettings::default())
            .expect("expected every committed zigbee model to load");

        assert_eq!(models.len(), sources.len());
    }

    #[test]
    fn every_committed_home_assistant_model_loads() {
        let sources = crate::lua::sources::load_directory(std::path::Path::new(
            "./config/lua/home_assistant",
        ))
        .expect("expected the committed home assistant models to be readable");

        let models = load_models("home_assistant", &sources, &LuaSettings::default())
            .expect("expected every committed home assistant model to load");

        assert!(models.contains_key("roborock"));
        assert!(models.contains_key("media_player"));
        assert_eq!(models.len(), sources.len());
    }
}
