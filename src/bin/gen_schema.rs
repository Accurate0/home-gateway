use home_gateway::device_registry::RawSensor;
use home_gateway::settings::{RawSettings, RawZigbeeModelProfile, WorkflowDefinition};
use schemars::Schema;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn write(dir: &Path, name: &str, schema: Schema) {
    let json = serde_json::to_string_pretty(&schema).expect("serialize schema");
    let out = dir.join(name);

    std::fs::write(&out, format!("{json}\n")).expect("write schema");
    eprintln!("wrote {}", out.display());
}

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

    write(
        &dir,
        "config.schema.json",
        schemars::schema_for!(RawSettings),
    );
    write(
        &dir,
        "workflows.schema.json",
        schemars::schema_for!(Vec<WorkflowDefinition>),
    );
    write(
        &dir,
        "devices.schema.json",
        schemars::schema_for!(Vec<RawSensor>),
    );
    write(
        &dir,
        "zigbee_models.schema.json",
        schemars::schema_for!(HashMap<String, RawZigbeeModelProfile>),
    );
}
