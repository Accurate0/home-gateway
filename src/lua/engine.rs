use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use mlua::{
    Function, HookTriggers, Lua, LuaSerdeExt, MultiValue, StdLib, Table, Value as LuaValue,
};
use tracing::{Instrument, Span};

use crate::settings::LuaSettings;
use crate::variables::{Node, VarType, Vars};

use super::bridge::{install_vars, returned_node};
use super::error::InstructionLimit;
use super::{
    CallTarget, LuaApiRegistry, LuaCallContext, LuaError, LuaSource, Script, builtin, bytecode,
};

pub(super) fn sandboxed_libs() -> StdLib {
    StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::OS | StdLib::UTF8
}

const STRIPPED_GLOBALS: [&str; 9] = [
    "collectgarbage",
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
    library: Arc<BTreeMap<String, Vec<u8>>>,
    scripts: Arc<BTreeMap<String, Vec<u8>>>,
    settings: LuaSettings,
}

impl LuaEngine {
    pub fn new(
        registry: LuaApiRegistry,
        library: BTreeMap<String, String>,
        scripts: BTreeMap<String, String>,
        settings: LuaSettings,
    ) -> Result<Self, String> {
        Ok(LuaEngine {
            registry,
            library: Arc::new(precompile("library", library)?),
            scripts: Arc::new(precompile("workflow script", scripts)?),
            settings,
        })
    }

    pub fn namespaces(&self) -> Vec<&'static str> {
        self.registry.namespaces()
    }

    pub async fn run_bool(
        &self,
        cx: &LuaCallContext,
        source: &LuaSource,
        vars: &Vars,
    ) -> Result<bool, LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;

        match self.eval_source(cx, &lua, source).await? {
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
        source: &LuaSource,
        vars: &Vars,
        declared: &BTreeMap<String, VarType>,
    ) -> Result<Node, LuaError> {
        let lua = self.prepare(cx, vars).map_err(LuaError::from_mlua)?;
        let value = self.eval_source(cx, &lua, source).await?;

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
        let value = self.eval_in(cx, &lua, script).await?;

        if value.is_nil() {
            return Ok(serde_json::Value::Null);
        }

        lua.from_value(value).map_err(LuaError::from_mlua)
    }

    pub async fn run_endpoint(
        &self,
        cx: &LuaCallContext,
        source: &LuaSource,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, LuaError> {
        let lua = self
            .prepare(cx, &Vars::default())
            .map_err(LuaError::from_mlua)?;

        let request = lua.to_value(request).map_err(LuaError::from_mlua)?;
        lua.globals()
            .set("request", request)
            .map_err(LuaError::from_mlua)?;

        let value = self.eval_source(cx, &lua, source).await?;

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

        self.eval_in(cx, &lua, script).await.map(|_| ())
    }

    async fn eval_source(
        &self,
        cx: &LuaCallContext,
        lua: &Lua,
        source: &LuaSource,
    ) -> Result<LuaValue, LuaError> {
        match source {
            LuaSource::Script { script } => self.eval_in(cx, lua, script).await,
            LuaSource::Call { call, args } => self.eval_call(cx, lua, call, args).await,
        }
    }

    async fn eval_call(
        &self,
        cx: &LuaCallContext,
        lua: &Lua,
        call: &CallTarget,
        args: &[serde_json::Value],
    ) -> Result<LuaValue, LuaError> {
        let timeout = self.settings.timeout();
        let started = Instant::now();

        let run = async {
            let source = self.scripts.get(call.script()).ok_or_else(|| {
                mlua::Error::external(format!(
                    "unknown lua workflow script `{}`; available: [{}]",
                    call.script(),
                    self.scripts.keys().cloned().collect::<Vec<_>>().join(", ")
                ))
            })?;

            let module: Table = bytecode::load(lua, call.script(), source)?
                .call_async(())
                .await?;

            let function: Option<Function> = module.get(call.function())?;
            let function = function.ok_or_else(|| {
                mlua::Error::external(format!("lua workflow script has no function `{call}`"))
            })?;

            let args = args
                .iter()
                .map(|arg| lua.to_value(arg))
                .collect::<mlua::Result<MultiValue>>()?;

            function.call_async::<LuaValue>(args).await
        };

        let span = eval_span(cx, &call.to_string());
        let result = match tokio::time::timeout(timeout, run)
            .instrument(span.clone())
            .await
        {
            Ok(evaluated) => evaluated.map_err(LuaError::from_mlua),
            Err(_) => Err(LuaError::Timeout(timeout)),
        };

        if let Err(e) = &result {
            crate::tracing_context::record_error(&span, &e.to_string());
        }

        record(&result, call.to_string(), started, lua.used_memory());

        result
    }

    async fn eval_in(
        &self,
        cx: &LuaCallContext,
        lua: &Lua,
        script: &Script,
    ) -> Result<LuaValue, LuaError> {
        let timeout = self.settings.timeout();
        let started = Instant::now();

        let span = eval_span(cx, "script");
        let result =
            match tokio::time::timeout(timeout, bytecode::eval(lua, "script", script.bytecode()))
                .instrument(span.clone())
                .await
            {
                Ok(evaluated) => evaluated.map_err(LuaError::from_mlua),
                Err(_) => Err(LuaError::Timeout(timeout)),
            };

        if let Err(e) = &result {
            crate::tracing_context::record_error(&span, &e.to_string());
        }

        record(&result, "script".to_owned(), started, lua.used_memory());

        result
    }

    fn prepare(&self, cx: &LuaCallContext, vars: &Vars) -> mlua::Result<Lua> {
        let started = Instant::now();

        let prepared = (|| {
            let lua = Lua::new_with(sandboxed_libs(), mlua::LuaOptions::default())?;

            sandbox(&lua)?;
            builtin::install(&lua, cx, &self.library)?;
            self.registry.install(&lua, cx)?;
            install_vars(&lua, vars)?;

            install_instruction_limit(&lua, self.settings.max_instructions)?;
            lua.set_memory_limit(self.settings.max_memory)?;

            Ok(lua)
        })();

        crate::metrics::record_lua_vm_setup("script", started.elapsed());

        prepared
    }
}

fn eval_span(cx: &LuaCallContext, source: &str) -> Span {
    tracing::info_span!(
        "lua.eval",
        otel.name = format!("lua eval: {source}"),
        lua.source = source,
        event_id = %cx.event_id,
        origin = %cx.origin,
        dry_run = cx.dry_run,
        otel.status_code = tracing::field::Empty,
        otel.status_message = tracing::field::Empty,
    )
}

fn precompile(
    label: &str,
    sources: BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut compiled = BTreeMap::new();

    for (name, source) in sources {
        let bytecode = bytecode::compile(&name, &source)
            .map_err(|error| format!("lua {label} `{name}` failed to compile: {error}"))?;

        compiled.insert(name, bytecode);
    }

    Ok(compiled)
}

fn record(
    result: &Result<LuaValue, LuaError>,
    source: String,
    started: Instant,
    memory_bytes: usize,
) {
    let outcome = match result {
        Ok(_) => "success",
        Err(LuaError::Timeout(_)) => "timeout",
        Err(LuaError::MemoryLimit) => "memory_limit",
        Err(_) => "error",
    };

    crate::metrics::record_lua(source, outcome, started.elapsed(), memory_bytes);
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

pub(super) fn sandbox(lua: &Lua) -> mlua::Result<()> {
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
            "collectgarbage",
            "io",
            "package",
            "require",
            "dofile",
            "loadfile",
            "load",
            "debug",
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
    fn the_memory_limit_stops_a_runaway_allocation() {
        let lua = bare_sandboxed_lua().expect("expected a sandboxed state");
        lua.set_memory_limit(16 * 1024 * 1024)
            .expect("expected the memory limit to apply");

        let error = lua
            .load("return string.rep('x', 64 * 1024 * 1024)")
            .exec()
            .expect_err("expected the allocation to be refused");

        assert!(
            matches!(LuaError::from_mlua(error), LuaError::MemoryLimit),
            "the allocation was not stopped by the memory limit"
        );
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
