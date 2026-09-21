use std::collections::BTreeMap;
use std::time::Instant;

use mlua::serde::SerializeOptions;
use mlua::{Function, Lua, LuaSerdeExt, Table, Value as LuaValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::settings::LuaSettings;

use super::LuaError;
use super::engine::{install_instruction_limit, sandbox, sandboxed_libs};

pub struct LuaDecoder {
    lua: Lua,
    kind: String,
    max_instructions: u32,
    modules: BTreeMap<String, Table>,
}

impl std::fmt::Debug for LuaDecoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaDecoder")
            .field("kind", &self.kind)
            .field("modules", &self.modules.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl LuaDecoder {
    pub fn load(
        kind: &str,
        sources: &BTreeMap<String, String>,
        settings: &LuaSettings,
    ) -> Result<Self, LuaError> {
        let started = Instant::now();

        let lua = Lua::new_with(sandboxed_libs(), mlua::LuaOptions::default())
            .map_err(LuaError::from_mlua)?;

        sandbox(&lua).map_err(LuaError::from_mlua)?;
        lua.set_memory_limit(settings.max_memory)
            .map_err(LuaError::from_mlua)?;
        install_instruction_limit(&lua, settings.max_instructions).map_err(LuaError::from_mlua)?;

        let mut modules = BTreeMap::new();

        for (name, source) in sources {
            let module: Table = lua
                .load(source.as_str())
                .set_name(format!("{kind}/{name}"))
                .eval()
                .map_err(LuaError::from_mlua)?;

            modules.insert(name.clone(), module);
        }

        crate::metrics::record_lua_vm_setup(kind, started.elapsed());

        Ok(LuaDecoder {
            lua,
            kind: kind.to_owned(),
            max_instructions: settings.max_instructions,
            modules,
        })
    }

    pub fn modules(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(String::as_str)
    }

    pub fn has_function(&self, module: &str, name: &str) -> Result<bool, LuaError> {
        let value: LuaValue = self
            .module(module)?
            .get(name)
            .map_err(LuaError::from_mlua)?;

        Ok(matches!(value, LuaValue::Function(_)))
    }

    pub fn field<T: DeserializeOwned>(
        &self,
        module: &str,
        name: &str,
    ) -> Result<Option<T>, LuaError> {
        let value: LuaValue = self
            .module(module)?
            .get(name)
            .map_err(LuaError::from_mlua)?;

        if value.is_nil() {
            return Ok(None);
        }

        self.lua
            .from_value(value)
            .map(Some)
            .map_err(LuaError::from_mlua)
    }

    pub fn call<I: Serialize, T: DeserializeOwned>(
        &self,
        module: &str,
        function: &str,
        input: &I,
    ) -> Result<T, LuaError> {
        let started = Instant::now();
        let result = self.invoke(module, function, input);

        let outcome = match &result {
            Ok(_) => "success",
            Err(LuaError::MemoryLimit) => "memory_limit",
            Err(_) => "error",
        };

        crate::metrics::record_lua(
            format!("{}/{module}.{function}", self.kind),
            outcome,
            started.elapsed(),
            self.lua.used_memory(),
        );

        result
    }

    fn invoke<I: Serialize, T: DeserializeOwned>(
        &self,
        module: &str,
        function: &str,
        input: &I,
    ) -> Result<T, LuaError> {
        let callable: Option<Function> = self
            .module(module)?
            .get(function)
            .map_err(LuaError::from_mlua)?;

        let Some(callable) = callable else {
            return Err(LuaError::Runtime(format!(
                "{}/{module} has no function `{function}`",
                self.kind
            )));
        };

        let options = SerializeOptions::new()
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false);

        let input = self
            .lua
            .to_value_with(input, options)
            .map_err(LuaError::from_mlua)?;

        install_instruction_limit(&self.lua, self.max_instructions).map_err(LuaError::from_mlua)?;

        let output: LuaValue = callable.call(input).map_err(LuaError::from_mlua)?;

        self.lua.from_value(output).map_err(LuaError::from_mlua)
    }

    fn module(&self, module: &str) -> Result<&Table, LuaError> {
        self.modules.get(module).ok_or_else(|| {
            LuaError::Runtime(format!(
                "unknown {} module `{module}`; available: [{}]",
                self.kind,
                self.modules.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::LuaDecoder;
    use crate::lua::LuaError;
    use crate::settings::LuaSettings;

    fn decoder(name: &str, source: &str) -> Result<LuaDecoder, LuaError> {
        let sources = BTreeMap::from([(name.to_owned(), source.to_owned())]);

        LuaDecoder::load("test", &sources, &LuaSettings::default())
    }

    #[test]
    fn a_module_function_maps_json_in_and_serde_out() {
        let decoder = decoder(
            "double",
            "return { run = function(p) return { value = p.value * 2, missing = p.absent == nil } end }",
        )
        .expect("expected the module to load");

        let output: serde_json::Value = decoder
            .call("double", "run", &json!({ "value": 21, "absent": null }))
            .expect("expected the call to succeed");

        assert_eq!(output, json!({ "value": 42, "missing": true }));
    }

    #[test]
    fn fields_read_back_as_typed_values() {
        let decoder = decoder("roles", "return { roles = { \"door\", \"battery\" } }")
            .expect("expected the module to load");

        let roles: Option<Vec<String>> = decoder.field("roles", "roles").expect("roles field");
        let absent: Option<Vec<String>> = decoder.field("roles", "other").expect("absent field");

        assert_eq!(roles, Some(vec!["door".to_owned(), "battery".to_owned()]));
        assert_eq!(absent, None);
        assert!(
            !decoder
                .has_function("roles", "roles")
                .expect("roles lookup")
        );
    }

    #[test]
    fn a_missing_function_is_an_error() {
        let decoder = decoder("empty", "return {}").expect("expected the module to load");

        let error = decoder
            .call::<_, serde_json::Value>("empty", "decode", &json!({}))
            .expect_err("expected no decode function");

        assert!(
            error.to_string().contains("has no function `decode`"),
            "{error}"
        );
    }

    #[test]
    fn a_runaway_loop_trips_the_instruction_limit_on_every_call() {
        let decoder = decoder("spin", "return { run = function() while true do end end }")
            .expect("expected the module to load");

        for _ in 0..2 {
            let error = decoder
                .call::<_, serde_json::Value>("spin", "run", &json!({}))
                .expect_err("expected the loop to be interrupted");

            assert!(matches!(error, LuaError::InstructionLimit(_)), "{error}");
        }
    }

    #[test]
    fn a_module_that_does_not_return_a_table_is_rejected() {
        let error = decoder("broken", "return 1").expect_err("expected a load error");

        assert!(matches!(error, LuaError::Runtime(_)), "{error}");
    }
}
