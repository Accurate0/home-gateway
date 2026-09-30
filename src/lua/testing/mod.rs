use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mlua::{Function, Lua, Table, Value as LuaValue};

use crate::settings::LuaSettings;

use super::LuaError;
use super::builtin::{json_decode, json_encode};
use super::engine::{install_instruction_limit, sandbox, sandboxed_libs};
use super::inspect::inspect;
use super::sources::load_directory;

mod captured_logs;
mod file;
mod run;

use captured_logs::CapturedLogs;

pub use file::LuaTestFile;
pub use run::LuaTestRun;

const PRELUDE: &str = include_str!("prelude.lua");

type Sources = Arc<BTreeMap<String, String>>;

#[derive(Clone)]
pub struct LuaTestRunner {
    root: Arc<PathBuf>,
    workflow: Sources,
    model: Sources,
}

impl LuaTestRunner {
    pub fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();

        Ok(LuaTestRunner {
            workflow: Arc::new(load_directory(&root.join("lib"))?),
            model: Arc::new(load_directory(&root.join("models/lib"))?),
            root: Arc::new(root),
        })
    }

    pub fn files(&self) -> anyhow::Result<Vec<LuaTestFile>> {
        let tests = self.root.join("tests");
        let mut paths = Vec::new();

        collect_lua_files(&tests, &mut paths)?;
        paths.sort();

        let files = paths
            .into_iter()
            .map(|path| {
                let name = path
                    .strip_prefix(&tests)
                    .unwrap_or(&path)
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");

                LuaTestFile { name, path }
            })
            .collect();

        Ok(files)
    }

    pub fn cases(&self, file: &LuaTestFile) -> Result<Vec<String>, LuaError> {
        let lua = self.vm().map_err(LuaError::from_mlua)?;
        let suite = self.suite(&lua, file).map_err(LuaError::from_mlua)?;

        let mut names = Vec::new();

        for pair in suite.pairs::<String, Function>() {
            let (name, _) = pair.map_err(LuaError::from_mlua)?;
            names.push(name);
        }

        if names.is_empty() {
            return Err(LuaError::Runtime(format!(
                "{} declares no test cases",
                file.name
            )));
        }

        names.sort();

        Ok(names)
    }

    pub fn run(&self, file: &LuaTestFile, case: &str) -> LuaTestRun {
        let lua = match self.vm() {
            Ok(lua) => lua,
            Err(e) => {
                return LuaTestRun {
                    result: Err(LuaError::from_mlua(e)),
                    logs: Vec::new(),
                };
            }
        };

        let result = self.call_case(&lua, file, case);

        let logs = lua
            .remove_app_data::<CapturedLogs>()
            .map(|captured| captured.lines)
            .unwrap_or_default();

        LuaTestRun { result, logs }
    }

    fn call_case(&self, lua: &Lua, file: &LuaTestFile, case: &str) -> Result<(), LuaError> {
        let suite = self.suite(lua, file).map_err(LuaError::from_mlua)?;
        let run: Function = suite.get(case).map_err(LuaError::from_mlua)?;

        install_instruction_limit(lua, LuaSettings::default().max_instructions)
            .map_err(LuaError::from_mlua)?;

        run.call::<()>(()).map_err(LuaError::from_mlua)
    }

    fn suite(&self, lua: &Lua, file: &LuaTestFile) -> mlua::Result<Table> {
        let source = std::fs::read_to_string(&file.path).map_err(mlua::Error::external)?;

        lua.load(source)
            .set_name(format!("@tests/{}.lua", file.name))
            .eval()
    }

    fn vm(&self) -> mlua::Result<Lua> {
        let settings = LuaSettings::default();
        let lua = Lua::new_with(sandboxed_libs(), mlua::LuaOptions::default())?;

        sandbox(&lua)?;
        lua.set_memory_limit(settings.max_memory)?;
        install_instruction_limit(&lua, settings.max_instructions)?;
        lua.set_app_data(CapturedLogs::default());

        lua.load(PRELUDE).set_name("@prelude.lua").exec()?;

        let globals = lua.globals();

        let gw: Table = globals.get("gw")?;
        gw.set("lib", library_loader(&lua, "lib", self.workflow.clone())?)?;
        gw.set(
            "log",
            lua.create_function(|lua, message: String| {
                if let Some(mut captured) = lua.app_data_mut::<CapturedLogs>() {
                    captured.lines.push(message);
                }

                Ok(())
            })?,
        )?;

        let runner = self.clone();
        let test: Table = globals.get("test")?;
        test.set(
            "load",
            lua.create_function(move |lua, path: String| runner.load_module(lua, &path))?,
        )?;
        test.set(
            "inspect",
            lua.create_function(|_, value: LuaValue| Ok(inspect(&value)))?,
        )?;

        let json = lua.create_table()?;
        json.set("decode", lua.create_function(json_decode)?)?;
        json.set("encode", lua.create_function(json_encode)?)?;
        globals.set("json", json)?;

        Ok(lua)
    }

    fn load_module(&self, lua: &Lua, path: &str) -> mlua::Result<LuaValue> {
        let source = std::fs::read_to_string(self.root.join(format!("{path}.lua")))
            .map_err(|e| mlua::Error::runtime(format!("cannot load `{path}`: {e}")))?;

        let chunk = lua.load(source).set_name(format!("@{path}.lua"));

        if !path.starts_with("models/") {
            return chunk.eval();
        }

        let gw = lua.create_table()?;
        gw.set(
            "lib",
            library_loader(lua, "models/lib", self.model.clone())?,
        )?;

        let env = lua.create_table()?;
        env.set("gw", gw)?;

        let meta = lua.create_table()?;
        meta.set("__index", lua.globals())?;
        env.set_metatable(Some(meta))?;

        chunk.set_environment(env).eval()
    }
}

fn collect_lua_files(directory: &Path, found: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_lua_files(&path, found)?;
        } else if path.extension().is_some_and(|ext| ext == "lua") {
            found.push(path);
        }
    }

    Ok(())
}

fn library_loader(lua: &Lua, kind: &'static str, library: Sources) -> mlua::Result<Function> {
    lua.create_function(move |lua, name: String| {
        let source = library.get(&name).ok_or_else(|| {
            mlua::Error::runtime(format!(
                "unknown {kind} library `{name}`; available: [{}]",
                library.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
        })?;

        lua.load(source.as_str())
            .set_name(format!("@{kind}/{name}.lua"))
            .eval::<LuaValue>()
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LuaTestRunner;

    fn scratch_root(tests: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("lua-tests-{}", uuid::Uuid::new_v4()));

        std::fs::create_dir_all(root.join("lib")).expect("lib dir");
        std::fs::create_dir_all(root.join("models/lib")).expect("model lib dir");

        for (name, source) in tests {
            let path = root.join("tests").join(format!("{name}.lua"));

            std::fs::create_dir_all(path.parent().expect("a parent")).expect("tests dir");
            std::fs::write(path, source).expect("write test");
        }

        root
    }

    #[test]
    fn nested_files_are_named_by_their_path_under_tests() {
        let root = scratch_root(&[
            ("b/second_test", "return { x = function() end }"),
            ("a_test", "return { x = function() end }"),
        ]);

        let runner = LuaTestRunner::new(&root).expect("runner");
        let names: Vec<String> = runner
            .files()
            .expect("files")
            .into_iter()
            .map(|file| file.name)
            .collect();

        assert_eq!(names, ["a_test", "b/second_test"]);
    }

    #[test]
    fn a_failing_case_reports_expected_and_actual() {
        let root = scratch_root(&[(
            "eq_test",
            r#"return { ["passes"] = function() test.eq({ 1 }, { 1 }) end, ["fails"] = function() test.eq("a", "b") end }"#,
        )]);

        let runner = LuaTestRunner::new(&root).expect("runner");
        let file = runner.files().expect("files").remove(0);

        assert_eq!(runner.cases(&file).expect("cases"), ["fails", "passes"]);
        assert!(runner.run(&file, "passes").result.is_ok());

        let error = runner
            .run(&file, "fails")
            .result
            .expect_err("a failure")
            .to_string();

        assert!(error.contains(r#"expected: "b""#), "{error}");
        assert!(error.contains(r#"actual: "a""#), "{error}");
    }

    #[test]
    fn cases_do_not_share_globals() {
        let root = scratch_root(&[(
            "isolation_test",
            "return { a = function() leaked = true end, b = function() assert(leaked == nil) end }",
        )]);

        let runner = LuaTestRunner::new(&root).expect("runner");
        let file = runner.files().expect("files").remove(0);

        assert!(runner.run(&file, "a").result.is_ok());
        assert!(runner.run(&file, "b").result.is_ok());
    }

    #[test]
    fn logs_are_captured_per_case() {
        let root = scratch_root(&[(
            "log_test",
            r#"return { a = function() gw.log("first") gw.log("second") end, b = function() end }"#,
        )]);

        let runner = LuaTestRunner::new(&root).expect("runner");
        let file = runner.files().expect("files").remove(0);

        assert_eq!(runner.run(&file, "a").logs, ["first", "second"]);
        assert!(runner.run(&file, "b").logs.is_empty());
    }

    #[test]
    fn a_file_without_cases_is_an_error() {
        let root = scratch_root(&[("empty_test", "return {}"), ("broken_test", "return 1")]);

        let runner = LuaTestRunner::new(&root).expect("runner");

        for file in runner.files().expect("files") {
            assert!(runner.cases(&file).is_err(), "{}", file.name);
        }
    }
}
