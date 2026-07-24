use std::collections::{HashMap, HashSet};
use std::path::Path;
use yaml_include::Transformer;

// Env-provided secrets are absent from the config files but required by the
// schema; inject dummies so structural validation passes.
const INJECTED_SECRETS: &[&str] = &[
    "api_key",
    "database_url",
    "mqtt_url",
    "mqtt_username",
    "mqtt_password",
    "unifi_webhook_secret",
    "android_app_webhook_secret",
];

fn trigger_vars(trigger_type: &str) -> Option<Vec<&'static str>> {
    Some(match trigger_type {
        "presence" => vec!["sensor", "present"],
        "door" => vec!["device", "open"],
        "switch" => vec!["device", "action"],
        "environment" => vec![
            "sensor",
            "temperature",
            "humidity",
            "pressure",
            "lux",
            "uv_index",
            "soil_moisture",
        ],
        "cron" => vec!["name"],
        "sun" => vec!["transition"],
        "mode" => vec!["mode", "active"],
        "home_assistant" => vec!["entity_id", "state"],
        "woolworths" => vec!["product_id", "name", "old_price", "new_price", "drop"],
        "device_battery" => vec!["device_id", "kind", "name", "battery_voltage"],
        _ => return None,
    })
}

fn placeholders(message: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = message;
    while let Some(start) = rest.find("${") {
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                names.push(&after[..end]);
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    names
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_dir = manifest_dir.join("config");
    println!("cargo:rerun-if-changed=config");

    let base = config_dir.join("base.yaml");
    let merged = Transformer::new(base.clone(), true)
        .unwrap_or_else(|e| panic!("failed to process includes in {}: {e}", base.display()))
        .to_string();

    let value: serde_json::Value =
        serde_yaml::from_str(&merged).expect("merged config is not valid YAML");

    validate_schema(&config_dir, value.clone());
    validate_semantics(&value);
}

fn validate_schema(config_dir: &Path, mut value: serde_json::Value) {
    let schema_path = config_dir.join("config.schema.json");
    let schema_str = std::fs::read_to_string(&schema_path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} (run `cargo run --bin gen_schema` to regenerate it)",
            schema_path.display()
        )
    });
    let schema: serde_json::Value =
        serde_json::from_str(&schema_str).expect("config.schema.json is not valid JSON");

    if let Some(obj) = value.as_object_mut() {
        for secret in INJECTED_SECRETS {
            obj.entry(*secret)
                .or_insert_with(|| serde_json::Value::String("x".to_owned()));
        }
    }

    let validator = jsonschema::validator_for(&schema).expect("config.schema.json is not valid");
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| format!("  at {}: {}", e.instance_path, e))
        .collect();
    if !errors.is_empty() {
        panic!("config failed schema validation:\n{}", errors.join("\n"));
    }
}

fn validate_semantics(value: &serde_json::Value) {
    let device_ids: HashSet<String> = value
        .get("devices")
        .and_then(|d| d.as_array())
        .map(|devices| {
            devices
                .iter()
                .filter_map(|d| d.get("id").and_then(|i| i.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // workflows: array of arrays (one inner array per included file)
    let workflows: Vec<&serde_json::Value> = value
        .get("workflows")
        .and_then(|w| w.as_array())
        .map(|outer| {
            outer
                .iter()
                .filter_map(|inner| inner.as_array())
                .flatten()
                .collect()
        })
        .unwrap_or_default();

    let names: HashSet<String> = workflows
        .iter()
        .filter_map(|w| w.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();

    let mut slugs: HashMap<String, String> = HashMap::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    for wf in &workflows {
        let name = wf
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");

        match wf.get("slug").and_then(|v| v.as_str()) {
            Some(slug) if !slug.trim().is_empty() => {
                if let Some(other) = slugs.insert(slug.to_owned(), name.to_owned()) {
                    panic!("duplicate workflow slug '{slug}' ('{other}' and '{name}')");
                }
            }
            _ => panic!("workflow '{name}' is missing a non-empty `slug`"),
        }

        if !seen_names.insert(name.to_owned()) {
            panic!("duplicate workflow name '{name}'");
        }

        let trigger_type = wf
            .get("on")
            .and_then(|o| o.get("type"))
            .and_then(|t| t.as_str());

        check_device_refs(wf.get("on"), &device_ids, name);
        check_device_refs(wf.get("when"), &device_ids, name);
        if let Some(run) = wf.get("run") {
            check_device_refs(Some(run), &device_ids, name);
            check_run_workflow_refs(run, &names, name);
            check_template_vars(run, trigger_type, name);
        }
    }
}

fn check_device_refs(node: Option<&serde_json::Value>, ids: &HashSet<String>, workflow: &str) {
    let Some(node) = node else { return };
    match node {
        serde_json::Value::Object(map) => {
            for key in ["device", "sensor"] {
                if let Some(reference) = map.get(key).and_then(|v| v.as_str())
                    && !ids.contains(reference)
                {
                    panic!(
                        "workflow '{workflow}': {key} `{reference}` is not a declared device id"
                    );
                }
            }
            for v in map.values() {
                check_device_refs(Some(v), ids, workflow);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                check_device_refs(Some(v), ids, workflow);
            }
        }
        _ => {}
    }
}

fn check_run_workflow_refs(run: &serde_json::Value, names: &HashSet<String>, workflow: &str) {
    let Some(steps) = run.as_array() else { return };
    for step in steps {
        let is_run = step.get("type").and_then(|t| t.as_str()) == Some("run_workflow");
        if is_run
            && let Some(target) = step.get("workflow").and_then(|w| w.as_str())
            && !names.contains(target)
        {
            panic!("workflow '{workflow}': run_workflow references unknown workflow '{target}'");
        }
        if let Some(nested) = step.get("run") {
            check_run_workflow_refs(nested, names, workflow);
        }
    }
}

fn check_template_vars(run: &serde_json::Value, trigger_type: Option<&str>, workflow: &str) {
    let Some(steps) = run.as_array() else { return };
    for step in steps {
        if step.get("type").and_then(|t| t.as_str()) == Some("notify")
            && let Some(message) = step.get("message").and_then(|m| m.as_str())
        {
            for var in placeholders(message) {
                let known = trigger_type
                    .and_then(trigger_vars)
                    .map(|vars| vars.contains(&var))
                    .unwrap_or(false);
                if !known {
                    let available = trigger_type
                        .and_then(trigger_vars)
                        .map(|v| v.join(", "))
                        .unwrap_or_else(|| "none (reusable workflow)".to_owned());
                    panic!(
                        "workflow '{workflow}': notify references unknown template var \
                         ${{{var}}}; trigger provides: [{available}]"
                    );
                }
            }
        }
        if let Some(nested) = step.get("run") {
            check_template_vars(nested, trigger_type, workflow);
        }
    }
}
