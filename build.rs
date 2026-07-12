use std::collections::HashMap;
use std::path::Path;

fn main() {
    let dir = Path::new("config/workflows");
    println!("cargo:rerun-if-changed=config/workflows");

    let mut slugs: HashMap<String, String> = HashMap::new();

    let entries = std::fs::read_dir(dir).expect("read config/workflows");
    for entry in entries {
        let path = entry.expect("dir entry").path();
        let is_yaml = path
            .extension()
            .map(|e| e == "yaml" || e == "yml")
            .unwrap_or(false);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !is_yaml || stem == "index" {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let yaml = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let workflows: Vec<serde_yaml::Value> = serde_yaml::from_str(&yaml)
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));

        for wf in workflows {
            let name = wf.get("name").and_then(|v| v.as_str()).unwrap_or("<unnamed>");
            let slug = wf.get("slug").and_then(|v| v.as_str());
            let slug = match slug {
                Some(s) if !s.trim().is_empty() => s.to_owned(),
                _ => panic!(
                    "workflow '{name}' in {} is missing a non-empty `slug`",
                    path.display()
                ),
            };
            if let Some(other) = slugs.insert(slug.clone(), path.display().to_string()) {
                panic!(
                    "duplicate workflow slug '{slug}' in {} and {}",
                    other,
                    path.display()
                );
            }
        }
    }
}
