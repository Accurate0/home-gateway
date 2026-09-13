use home_gateway::device_registry::RawSensor;
use home_gateway::settings::{RawSettings, RawZigbeeModelProfile, WorkflowDefinition};
use schemars::Schema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const ENV_PROVIDED_ROOT: &[&str] = &["api_key", "database_url"];
const ENV_PROVIDED_MQTT: &[&str] = &["url", "username", "password"];

fn write(dir: &Path, name: &str, value: &Value) {
    let json = serde_json::to_string_pretty(value).expect("serialize schema");
    let out = dir.join(name);

    std::fs::write(&out, format!("{json}\n")).expect("write schema");
    eprintln!("wrote {}", out.display());
}

fn to_value(schema: Schema) -> Value {
    serde_json::to_value(schema).expect("schema to value")
}

fn drop_required(node: &mut Value, names: &[&str]) {
    if let Some(required) = node.get_mut("required").and_then(Value::as_array_mut) {
        required.retain(|entry| !names.iter().any(|name| entry == name));
    }
}

fn editor_schema(mut schema: Value) -> Value {
    drop_required(&mut schema, ENV_PROVIDED_ROOT);

    if let Some(mqtt) = schema.pointer_mut("/$defs/MqttSettings") {
        drop_required(mqtt, ENV_PROVIDED_MQTT);
    }

    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        for property in properties.values_mut() {
            let original = property.take();
            *property = json!({ "anyOf": [original, { "type": "string" }] });
        }
    }

    schema
}

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("schemas");

    std::fs::create_dir_all(&dir).expect("create schema dir");

    let config = to_value(schemars::schema_for!(RawSettings));

    write(&dir, "config.schema.json", &config);
    write(&dir, "config.editor.schema.json", &editor_schema(config));
    write(
        &dir,
        "workflows.schema.json",
        &to_value(schemars::schema_for!(Vec<WorkflowDefinition>)),
    );
    write(
        &dir,
        "devices.schema.json",
        &to_value(schemars::schema_for!(Vec<RawSensor>)),
    );
    write(
        &dir,
        "zigbee_models.schema.json",
        &to_value(schemars::schema_for!(HashMap<String, RawZigbeeModelProfile>)),
    );

    let lua_types = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("lua")
        .join("types")
        .join("gateway.lua");

    std::fs::write(&lua_types, home_gateway::startup::lua::type_definitions())
        .expect("write lua types");
    eprintln!("wrote {}", lua_types.display());
}
