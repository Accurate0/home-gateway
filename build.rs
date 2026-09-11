use std::collections::{HashMap, HashSet};
use std::path::Path;
use yaml_include::Transformer;

// Env-provided secrets are absent from the config files but required by the
// schema; inject dummies so structural validation passes.
const INJECTED_SECRETS: &[&str] = &[
    "api_key",
    "database_url",
    "mqtt.url",
    "mqtt.username",
    "mqtt.password",
];

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

    for secret in INJECTED_SECRETS {
        inject_secret(&mut value, secret);
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

fn inject_secret(value: &mut serde_json::Value, path: &str) {
    let (parents, leaf) = match path.rsplit_once('.') {
        Some((parents, leaf)) => (parents.split('.').collect::<Vec<_>>(), leaf),
        None => (Vec::new(), path),
    };

    let mut node = value;

    for parent in parents {
        let Some(obj) = node.as_object_mut() else {
            return;
        };

        node = obj
            .entry(parent)
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    }

    if let Some(obj) = node.as_object_mut() {
        obj.entry(leaf)
            .or_insert_with(|| serde_json::Value::String("x".to_owned()));
    }
}

fn validate_semantics(value: &serde_json::Value) {
    let device_ids: HashSet<String> = value
        .get("devices")
        .and_then(|d| d.as_array())
        .map(|outer| {
            outer
                .iter()
                .filter_map(|inner| inner.as_array())
                .flatten()
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

        check_device_refs(wf.get("on"), &device_ids, name);
        check_device_refs(wf.get("when"), &device_ids, name);
        if let Some(run) = wf.get("run") {
            check_device_refs(Some(run), &device_ids, name);
            check_run_workflow_refs(run, &names, name);
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
