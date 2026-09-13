use std::collections::BTreeMap;
use std::sync::Arc;

use mlua::{HookTriggers, Lua, LuaSerdeExt, StdLib, Table, Value as LuaValue};

use crate::settings::LuaSettings;
use crate::variables::{Node, VarType, Vars};

use super::bridge::{install_vars, returned_node};
use super::error::InstructionLimit;
use super::{LuaApiRegistry, LuaCallContext, LuaError, Script, builtin};

fn sandboxed_libs() -> StdLib {
    StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::OS | StdLib::UTF8
}

const STRIPPED_GLOBALS: [&str; 8] = [
    "io",
    "package",
    "require",
    "dofile",
    "loadfile",
    "load",
    "loadstring",
    "debug",
];

const ALLOWED_OS: [&str; 3] = ["time", "date", "clock"];

#[derive(Clone, Default)]
pub struct LuaEngine {
    registry: LuaApiRegistry,
    library: Arc<BTreeMap<String, String>>,
    settings: LuaSettings,
}

impl LuaEngine {
    pub fn new(
        registry: LuaApiRegistry,
        library: BTreeMap<String, String>,
        settings: LuaSettings,
    ) -> Self {
        LuaEngine {
            registry,
            library: Arc::new(library),
            settings,
        }
    }

    pub fn namespaces(&self) -> Vec<&'static str> {
        self.registry.namespaces()
    }

    pub async fn run_bool(
        &self,
        cx: &LuaCallContext,
        script: &Script,
        vars: &Vars,
    ) -> Result<bool, LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;

        match self.eval_in(&lua, script).await? {
            LuaValue::Boolean(flag) => Ok(flag),
            other => Err(LuaError::ReturnType {
                got: other.type_name(),
                expected: "boolean",
            }),
        }
    }

    pub async fn run_returning(
        &self,
        cx: &LuaCallContext,
        script: &Script,
        vars: &Vars,
        declared: &BTreeMap<String, VarType>,
    ) -> Result<Node, LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;
        let value = self.eval_in(&lua, script).await?;

        if declared.is_empty() {
            return Ok(Node::empty());
        }

        match value {
            LuaValue::Table(table) => returned_node(&table, declared),
            other => Err(LuaError::ReturnType {
                got: other.type_name(),
                expected: "table",
            }),
        }
    }

    pub async fn run_json(
        &self,
        cx: &LuaCallContext,
        script: &Script,
        vars: &Vars,
    ) -> Result<serde_json::Value, LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;
        let value = self.eval_in(&lua, script).await?;

        if value.is_nil() {
            return Ok(serde_json::Value::Null);
        }

        lua.from_value(value).map_err(LuaError::from_mlua)
    }

    pub async fn run_unit(
        &self,
        cx: &LuaCallContext,
        script: &Script,
        vars: &Vars,
    ) -> Result<(), LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;

        self.eval_in(&lua, script).await.map(|_| ())
    }

    async fn eval_in(&self, lua: &Lua, script: &Script) -> Result<LuaValue, LuaError> {
        let timeout = self.settings.timeout();
        let chunk = script.raw().to_owned();

        let evaluated = tokio::time::timeout(
            timeout,
            lua.load(chunk).set_name("script").eval_async::<LuaValue>(),
        )
        .await
        .map_err(|_| LuaError::Timeout(timeout))?;

        evaluated.map_err(LuaError::from_mlua)
    }

    fn prepare(&self, cx: &LuaCallContext, vars: &Vars) -> mlua::Result<Lua> {
        let lua = Lua::new_with(sandboxed_libs(), mlua::LuaOptions::default())?;

        sandbox(&lua)?;
        builtin::install(&lua, cx, &self.library)?;
        self.registry.install(&lua, cx)?;
        install_vars(&lua, vars)?;

        install_instruction_limit(&lua, self.settings.max_instructions)?;

        Ok(lua)
    }
}

#[cfg(test)]
pub(super) fn bare_sandboxed_lua() -> mlua::Result<Lua> {
    let lua = Lua::new_with(sandboxed_libs(), mlua::LuaOptions::default())?;

    sandbox(&lua)?;

    Ok(lua)
}

pub(super) fn install_instruction_limit(lua: &Lua, limit: u32) -> mlua::Result<()> {
    let triggers = HookTriggers::new().every_nth_instruction(limit);
    let trip =
        move |_: &Lua, _: &mlua::debug::Debug| Err(mlua::Error::external(InstructionLimit(limit)));

    lua.set_global_hook(triggers, trip)?;
    lua.set_hook(triggers, trip)
}

fn sandbox(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    for name in STRIPPED_GLOBALS {
        globals.set(name, LuaValue::Nil)?;
    }

    let os: Table = globals.get("os")?;
    let trimmed = lua.create_table()?;

    for name in ALLOWED_OS {
        let function: LuaValue = os.get(name)?;
        trimmed.set(name, function)?;
    }

    globals.set("os", trimmed)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use mlua::Value as LuaValue;

    use super::super::LuaError;
    use super::{bare_sandboxed_lua, install_instruction_limit};

    #[test]
    fn the_sandbox_strips_host_access() {
        let lua = bare_sandboxed_lua().expect("expected a sandboxed state");

        for global in [
            "io", "package", "require", "dofile", "loadfile", "load", "debug",
        ] {
            let value: LuaValue = lua
                .globals()
                .get(global)
                .expect("expected the global to be readable");

            assert!(value.is_nil(), "`{global}` is still reachable from lua");
        }
    }

    #[test]
    fn the_sandbox_keeps_os_time_but_drops_os_execute() {
        let lua = bare_sandboxed_lua().expect("expected a sandboxed state");

        let time: LuaValue = lua
            .load("return os.time")
            .eval()
            .expect("expected os.time to be readable");
        let execute: LuaValue = lua
            .load("return os.execute")
            .eval()
            .expect("expected os.execute to be readable");

        assert!(!time.is_nil(), "os.time was stripped");
        assert!(execute.is_nil(), "os.execute is still reachable from lua");
    }

    #[test]
    fn the_instruction_hook_stops_an_infinite_loop() {
        let lua = bare_sandboxed_lua().expect("expected a sandboxed state");
        install_instruction_limit(&lua, 10_000).expect("expected the hook to install");

        let error = lua
            .load("while true do end")
            .exec()
            .expect_err("expected the loop to be interrupted");

        assert!(
            matches!(
                LuaError::from_mlua(error),
                LuaError::InstructionLimit(10_000)
            ),
            "the loop was not stopped by the instruction limit"
        );
    }

    #[tokio::test]
    async fn the_instruction_hook_also_stops_a_loop_on_the_async_path() {
        let lua = bare_sandboxed_lua().expect("expected a sandboxed state");
        install_instruction_limit(&lua, 10_000).expect("expected the hook to install");

        let error = lua
            .load("while true do end")
            .exec_async()
            .await
            .expect_err("expected the loop to be interrupted");

        assert!(
            matches!(
                LuaError::from_mlua(error),
                LuaError::InstructionLimit(10_000)
            ),
            "an async chunk runs on its own lua thread and must inherit the hook"
        );
    }
}
