use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::settings::SettingsContainer;

pub fn load_configured(relative: Option<&Path>) -> anyhow::Result<BTreeMap<String, String>> {
    let Some(relative) = relative else {
        return Ok(BTreeMap::new());
    };

    let candidates = [
        ("override", SettingsContainer::override_dir().join(relative)),
        ("baked-in", SettingsContainer::baked_dir().join(relative)),
    ];

    load_directory(&resolve_directory(&candidates)?)
}

fn resolve_directory(candidates: &[(&str, PathBuf)]) -> anyhow::Result<PathBuf> {
    for (source, candidate) in candidates {
        if !candidate.is_dir() {
            tracing::warn!(
                "lua library directory {} does not exist",
                candidate.display()
            );

            continue;
        }

        tracing::info!(
            "loading lua library from {} ({source})",
            candidate.display()
        );

        return Ok(candidate.clone());
    }

    anyhow::bail!(
        "no lua library directory found in [{}]",
        candidates
            .iter()
            .map(|(_, candidate)| candidate.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn load_directory(directory: &Path) -> anyhow::Result<BTreeMap<String, String>> {
    let mut library = BTreeMap::new();

    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();

        if path.extension().is_none_or(|ext| ext != "lua") {
            continue;
        }

        let Some(name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };

        library.insert(name.to_owned(), std::fs::read_to_string(&path)?);
    }

    Ok(library)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{load_directory, resolve_directory};

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lua-library-{}-{name}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("expected the scratch dir to be created");

        dir
    }

    #[test]
    fn the_override_library_wins_when_present() {
        let override_dir = scratch_dir("override");
        let baked_dir = scratch_dir("baked");

        let resolved =
            resolve_directory(&[("override", override_dir.clone()), ("baked-in", baked_dir)])
                .expect("expected a library directory");

        assert_eq!(resolved, override_dir);
    }

    #[test]
    fn the_baked_library_is_used_when_the_override_is_missing() {
        let baked_dir = scratch_dir("baked");
        let missing =
            std::env::temp_dir().join(format!("lua-library-missing-{}", uuid::Uuid::new_v4()));

        let resolved = resolve_directory(&[("override", missing), ("baked-in", baked_dir.clone())])
            .expect("expected a library directory");

        assert_eq!(resolved, baked_dir);
    }

    #[test]
    fn no_library_directory_is_an_error() {
        let missing =
            std::env::temp_dir().join(format!("lua-library-missing-{}", uuid::Uuid::new_v4()));

        let error =
            resolve_directory(&[("override", missing)]).expect_err("expected no library directory");

        assert!(error.to_string().contains("no lua library directory"));
    }

    #[test]
    fn only_lua_files_are_loaded_keyed_by_stem() {
        let dir = scratch_dir("load");
        std::fs::write(dir.join("plug.lua"), "return {}").expect("write lua");
        std::fs::write(dir.join("notes.txt"), "ignored").expect("write txt");

        let loaded = load_directory(&dir).expect("expected the directory to load");

        assert_eq!(loaded.keys().collect::<Vec<_>>(), ["plug"]);
    }
}
