use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::actors::devices::door_events::lua::DoorLua;
use crate::actors::devices::environment_sensor::lua::EnvironmentLua;
use crate::actors::devices::light::lua::LightLua;
use crate::actors::devices::presence_sensor::lua::PresenceLua;
use crate::actors::devices::smart_switch::lua::SwitchLua;
use crate::actors::integrations::holidays::lua::HolidaysLua;
use crate::actors::integrations::solar::lua::SolarLua;
use crate::actors::sun::lua::SunLua;
use crate::actors::system::push::lua::NotifyLua;
use crate::actors::workflows::lua::WorkflowLua;
use crate::integrations::fuelwatch::variables::FuelwatchVariables;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::home_assistant::lua::HomeAssistantLua;
use crate::integrations::mqtt::lua::MqttLua;
use crate::integrations::willyweather::variables::WillyweatherVariables;
use crate::lua::{
    LuaApiRegistry, LuaEngine, LuaField, LuaNamespace, LuaType, builtin, schema, typegen,
};
use crate::settings::{Settings, SettingsContainer};
use crate::state::HandleRegistry;

const GLOBALS: &[LuaField] = &[
    LuaField {
        name: "event",
        ty: LuaType::Map(&LuaType::Any),
    },
    LuaField {
        name: "input",
        ty: LuaType::Map(&LuaType::Any),
    },
    LuaField {
        name: "lua",
        ty: LuaType::Map(&LuaType::Any),
    },
    LuaField {
        name: "willyweather",
        ty: LuaType::Schema(schema::<WillyweatherVariables>),
    },
    LuaField {
        name: "fuelwatch",
        ty: LuaType::Schema(schema::<FuelwatchVariables>),
    },
];

pub fn registry(home_assistant: bool) -> LuaApiRegistry {
    LuaApiRegistry::builder()
        .insert(LightLua)
        .insert(SwitchLua)
        .insert(PresenceLua)
        .insert(DoorLua)
        .insert(EnvironmentLua)
        .insert(NotifyLua)
        .insert(MqttLua)
        .insert(WorkflowLua)
        .insert(SunLua)
        .insert(SolarLua)
        .insert(HolidaysLua)
        .insert_optional(home_assistant.then_some(HomeAssistantLua))
        .build()
}

pub fn type_definitions() -> String {
    let mut namespaces = vec![
        LuaNamespace {
            name: "gw",
            fields: builtin::GW_FIELDS,
            functions: builtin::GW_FUNCTIONS,
        },
        LuaNamespace {
            name: "json",
            fields: &[],
            functions: builtin::JSON_FUNCTIONS,
        },
    ];

    namespaces.extend(registry(true).api());

    typegen::render(&namespaces, GLOBALS)
}

pub fn build(settings: &Settings, handles: &HandleRegistry) -> anyhow::Result<LuaEngine> {
    let registry = registry(handles.contains::<HomeAssistant>());

    let library = load_configured(settings.lua.library.as_deref())?;
    let scripts = load_configured(settings.lua.scripts.as_deref())?;

    tracing::info!(
        "lua namespaces: [{}], library: [{}], workflow scripts: [{}]",
        registry.namespaces().join(", "),
        library.keys().cloned().collect::<Vec<_>>().join(", "),
        scripts.keys().cloned().collect::<Vec<_>>().join(", ")
    );

    Ok(LuaEngine::new(
        registry,
        library,
        scripts,
        settings.lua.clone(),
    ))
}

fn load_configured(relative: Option<&Path>) -> anyhow::Result<BTreeMap<String, String>> {
    let Some(relative) = relative else {
        return Ok(BTreeMap::new());
    };

    let candidates = [
        ("override", SettingsContainer::override_dir().join(relative)),
        ("baked-in", SettingsContainer::baked_dir().join(relative)),
    ];

    load_library(&resolve_library(&candidates)?)
}

fn resolve_library(candidates: &[(&str, PathBuf)]) -> anyhow::Result<PathBuf> {
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

fn load_library(directory: &Path) -> anyhow::Result<BTreeMap<String, String>> {
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

    use std::collections::BTreeSet;

    use super::{registry, resolve_library, type_definitions};

    #[test]
    fn the_committed_lua_types_are_up_to_date() {
        let committed = include_str!("../../config/lua/types/gateway.lua");

        assert!(
            type_definitions() == committed,
            "config/lua/types/gateway.lua is stale: run `cargo run --bin gen_schema`"
        );
    }

    #[test]
    fn every_namespace_declares_unique_function_names() {
        for namespace in registry(true).api() {
            let mut seen = BTreeSet::new();

            for function in namespace.functions {
                assert!(
                    seen.insert(function.name),
                    "`{}.{}` is declared twice",
                    namespace.name,
                    function.name
                );
            }
        }
    }

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
            resolve_library(&[("override", override_dir.clone()), ("baked-in", baked_dir)])
                .expect("expected a library directory");

        assert_eq!(resolved, override_dir);
    }

    #[test]
    fn the_baked_library_is_used_when_the_override_is_missing() {
        let baked_dir = scratch_dir("baked");
        let missing =
            std::env::temp_dir().join(format!("lua-library-missing-{}", uuid::Uuid::new_v4()));

        let resolved = resolve_library(&[("override", missing), ("baked-in", baked_dir.clone())])
            .expect("expected a library directory");

        assert_eq!(resolved, baked_dir);
    }

    #[test]
    fn no_library_directory_is_an_error() {
        let missing =
            std::env::temp_dir().join(format!("lua-library-missing-{}", uuid::Uuid::new_v4()));

        let error =
            resolve_library(&[("override", missing)]).expect_err("expected no library directory");

        assert!(error.to_string().contains("no lua library directory"));
    }
}
