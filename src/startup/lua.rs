use crate::actors::alarm::lua::AlarmLua;
use crate::actors::devices::door_events::lua::DoorLua;
use crate::actors::devices::environment_sensor::lua::EnvironmentLua;
use crate::actors::devices::light::lua::LightLua;
use crate::actors::devices::media_player::lua::MediaLua;
use crate::actors::devices::presence_sensor::lua::PresenceLua;
use crate::actors::devices::robot_vacuum::lua::VacuumLua;
use crate::actors::devices::smart_switch::lua::SwitchLua;
use crate::actors::integrations::solar::lua::SolarLua;
use crate::actors::integrations::synergy::lua::EnergyLua;
use crate::actors::integrations::unifi::lua::UnifiLua;
use crate::actors::sun::lua::SunLua;
use crate::actors::system::battery::lua::BatteryLua;
use crate::actors::system::push::lua::NotifyLua;
use crate::actors::vacation::lua::VacationLua;
use crate::actors::workflows::lua::WorkflowLua;
use crate::device_registry::lua::DeviceLua;
use crate::integrations::fuelwatch::variables::FuelwatchVariables;
use crate::integrations::holidays::lua::HolidaysLua;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::home_assistant::lua::HomeAssistantLua;
use crate::integrations::mqtt::lua::MqttLua;
use crate::integrations::s3::lua::S3Lua;
use crate::integrations::transperth::lua::TransperthLua;
use crate::integrations::willyweather::variables::WillyweatherVariables;
use crate::integrations::woolworths::lua::WoolworthsLua;
use crate::lua::flag::FlagLua;
use crate::lua::regex::RegexLua;
use crate::lua::sources::load_configured;
use crate::lua::state::StateLua;
use crate::lua::time::TimeLua;
use crate::lua::{
    LuaApiRegistry, LuaEngine, LuaField, LuaNamespace, LuaType, builtin, schema, typegen,
};
use crate::settings::Settings;
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
        .insert(StateLua)
        .insert(TimeLua)
        .insert(DeviceLua)
        .insert(BatteryLua)
        .insert(VacuumLua)
        .insert(EnergyLua)
        .insert(WoolworthsLua)
        .insert(TransperthLua)
        .insert(RegexLua)
        .insert(AlarmLua)
        .insert(VacationLua)
        .insert(UnifiLua)
        .insert(MediaLua)
        .insert(FlagLua)
        .insert(S3Lua)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{registry, type_definitions};

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
}
