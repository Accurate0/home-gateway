use home_gateway::settings::RawSettings;
use std::path::PathBuf;

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema = schemars::schema_for!(RawSettings);
    let json = serde_json::to_string_pretty(&schema).expect("serialize schema");
    let out = base.join("config").join("config.schema.json");
    std::fs::write(&out, format!("{json}\n")).expect("write schema");
    eprintln!("wrote {}", out.display());
}
