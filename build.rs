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

fn context_vars(source: &str) -> Option<Vec<&'static str>> {
    Some(match source {
        "fuelwatch" => vec![
            "fuel_price",
            "fuel_brand",
            "fuel_name",
            "fuel_suburb",
            "fuel_address",
        ],
        _ => return None,
    })
}

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
        "fuelwatch" => vec![
            "change",
            "site_id",
            "name",
            "brand",
            "suburb",
            "address",
            "old_price",
            "new_price",
            "drop",
        ],
        "device_battery" => vec![
            "device_id",
            "kind",
            "name",
            "battery_voltage",
            "battery_percent",
        ],
        "jellyfin" => vec![
            "state",
            "session_id",
            "user",
            "device",
            "client",
            "item",
            "item_type",
            "series",
            "season",
            "episode",
            "position",
            "runtime",
            "play_method",
        ],
        "media_player" => vec![
            "device",
            "name",
            "room",
            "state",
            "entity_state",
            "app",
            "source",
            "item",
            "series",
            "item_type",
            "season",
            "episode",
            "position",
            "duration",
            "volume",
            "muted",
        ],
        "solar" => vec!["current", "avg_15m", "avg_1h", "avg_3h"],
        "weather" => vec![
            "source",
            "temperature",
            "feels_like",
            "humidity",
            "wind_speed",
            "gust_speed",
            "max_gust_speed",
            "rain_since_9am",
            "uv",
            "max_temp",
            "min_temp",
            "today_max_temp",
            "today_min_temp",
            "today_uv_max",
            "tomorrow_max_temp",
            "tomorrow_min_temp",
            "tomorrow_uv_max",
        ],
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

fn git_short_sha(manifest_dir: &Path) -> Option<String> {
    println!("cargo:rerun-if-env-changed=GIT_SHA");

    if let Ok(sha) = std::env::var("GIT_SHA") {
        let sha = sha.trim().to_owned();
        if !sha.is_empty() {
            return Some(sha.chars().take(7).collect());
        }
    }

    let git_dir = manifest_dir.join(".git");
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());

    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();

    let sha = match head.strip_prefix("ref: ") {
        Some(reference) => {
            let ref_path = git_dir.join(reference);
            println!("cargo:rerun-if-changed={}", ref_path.display());

            match std::fs::read_to_string(&ref_path) {
                Ok(sha) => sha.trim().to_owned(),
                Err(_) => {
                    let packed = std::fs::read_to_string(git_dir.join("packed-refs")).ok()?;
                    packed
                        .lines()
                        .filter(|line| !line.starts_with(['#', '^']))
                        .find_map(|line| {
                            let (sha, name) = line.split_once(' ')?;
                            (name == reference).then(|| sha.trim().to_owned())
                        })?
                }
            }
        }
        None => head.to_owned(),
    };

    let sha: String = sha.chars().take(7).collect();

    (sha.len() == 7 && sha.chars().all(|c| c.is_ascii_hexdigit())).then_some(sha)
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let version = match git_short_sha(manifest_dir) {
        Some(sha) => format!("{}-{sha}", env!("CARGO_PKG_VERSION")),
        None => env!("CARGO_PKG_VERSION").to_owned(),
    };
    println!("cargo:rustc-env=HOME_GATEWAY_VERSION={version}");

    println!("cargo:rerun-if-env-changed=SKIP_SCHEMA_VALIDATION");
    if std::env::var_os("SKIP_SCHEMA_VALIDATION").is_some() {
        return;
    }

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
        .map(|e| format!("  at {}: {}", e.instance_path(), e))
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

        let context: Vec<&'static str> = wf
            .get("context")
            .and_then(|c| c.as_array())
            .into_iter()
            .flatten()
            .flat_map(|source| {
                let source = source.as_str().unwrap_or("<non-string>");
                context_vars(source).unwrap_or_else(|| {
                    panic!("workflow '{name}': unknown context source '{source}'")
                })
            })
            .collect();

        let available = trigger_type.and_then(trigger_vars).map(|mut vars| {
            vars.extend(context.iter().copied());
            vars
        });

        check_device_refs(wf.get("on"), &device_ids, name);
        check_device_refs(wf.get("when"), &device_ids, name);
        if let Some(run) = wf.get("run") {
            check_device_refs(Some(run), &device_ids, name);
            check_run_workflow_refs(run, &names, name);
            check_template_vars(run, available.as_deref(), name);
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

fn templated_strings(step: &serde_json::Value) -> Vec<&str> {
    let fields: &[&str] = match step.get("type").and_then(|t| t.as_str()) {
        Some("notify") => &["message", "title"],
        Some("mqtt_publish") => &["topic", "payload"],
        Some("http") => &["url", "body"],
        _ => return Vec::new(),
    };

    let headers = step
        .get("headers")
        .and_then(|h| h.as_object())
        .into_iter()
        .flat_map(|h| h.values());

    fields
        .iter()
        .filter_map(|field| step.get(*field))
        .chain(headers)
        .filter_map(|value| value.as_str())
        .collect()
}

fn check_template_vars(run: &serde_json::Value, available: Option<&[&str]>, workflow: &str) {
    let Some(steps) = run.as_array() else { return };
    for step in steps {
        let kind = step
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("<untyped>");

        for template in templated_strings(step) {
            for var in placeholders(template) {
                let known = available.is_some_and(|vars| vars.contains(&var));
                if !known {
                    let listed = available
                        .map(|v| v.join(", "))
                        .unwrap_or_else(|| "none (reusable workflow)".to_owned());
                    panic!(
                        "workflow '{workflow}': {kind} references unknown template var \
                         ${{{var}}}; trigger and context provide: [{listed}]"
                    );
                }
            }
        }
        if let Some(nested) = step.get("run") {
            check_template_vars(nested, available, workflow);
        }
    }
}
