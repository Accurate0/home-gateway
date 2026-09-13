use schemars::JsonSchema;
use serde::Deserialize;

use super::Script;

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RawLuaSource {
    Script {
        script: Script,
    },
    Call {
        call: String,
        #[serde(default)]
        args: Vec<serde_json::Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "RawLuaSource")]
#[schemars(with = "RawLuaSource")]
pub struct LuaSource {
    script: Script,
    call: Option<String>,
}

impl LuaSource {
    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn summary(&self) -> String {
        match &self.call {
            Some(call) => format!("call {call}"),
            None => self.script.summary(),
        }
    }
}

impl TryFrom<RawLuaSource> for LuaSource {
    type Error = String;

    fn try_from(raw: RawLuaSource) -> Result<Self, Self::Error> {
        match raw {
            RawLuaSource::Script { script } => Ok(LuaSource { script, call: None }),
            RawLuaSource::Call { call, args } => {
                let script = call_script(&call, &args)?;

                Ok(LuaSource {
                    script,
                    call: Some(call),
                })
            }
        }
    }
}

fn call_script(call: &str, args: &[serde_json::Value]) -> Result<Script, String> {
    let Some((library, function)) = call
        .split_once('.')
        .filter(|(library, function)| is_identifier(library) && is_identifier(function))
    else {
        return Err(format!(
            "`{call}` must be written as `library.function` using lua identifiers"
        ));
    };

    let args = serde_json::to_string(args).map_err(|error| error.to_string())?;

    Script::parse(&format!(
        "return gw.lib(\"{library}\").{function}(table.unpack(json.decode({args:?})))"
    ))
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();

    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use mlua::{Lua, LuaSerdeExt, Table};

    use super::{LuaSource, RawLuaSource};

    fn call(call: &str, args: serde_json::Value) -> Result<LuaSource, String> {
        LuaSource::try_from(RawLuaSource::Call {
            call: call.to_owned(),
            args: serde_json::from_value(args).expect("expected an args array"),
        })
    }

    fn echo_lua() -> Lua {
        let lua = Lua::new();

        lua.load(
            "gw = { lib = function(name)
                return { echo = function(...) return { library = name, args = { ... } } end }
            end }",
        )
        .exec()
        .expect("expected the stub library to load");

        let json = lua.create_table().expect("expected a json table");
        json.set(
            "decode",
            lua.create_function(|lua, raw: String| {
                let parsed: serde_json::Value =
                    serde_json::from_str(&raw).map_err(mlua::Error::external)?;

                lua.to_value(&parsed)
            })
            .expect("expected json.decode"),
        )
        .expect("expected json.decode to be set");
        lua.globals()
            .set("json", json)
            .expect("expected json to be set");

        lua
    }

    #[test]
    fn a_call_invokes_the_library_function_with_its_args() {
        let source = call(
            "lights.echo",
            serde_json::json!([["kitchen", "hall"], "say \"hi\"\n", 3]),
        )
        .expect("expected the call to parse");

        let lua = echo_lua();
        let result: Table = lua
            .load(source.script().raw())
            .eval()
            .expect("expected the call to run");
        let result: serde_json::Value = lua
            .from_value(mlua::Value::Table(result))
            .expect("expected a json result");

        assert_eq!(
            result,
            serde_json::json!({
                "library": "lights",
                "args": [["kitchen", "hall"], "say \"hi\"\n", 3],
            })
        );
        assert_eq!(source.summary(), "call lights.echo");
    }

    #[test]
    fn a_call_must_name_a_library_and_a_function() {
        for bad in [
            "lights",
            "lights.",
            ".all",
            "lights.all.on",
            "lights.all()",
            "1ights.all",
        ] {
            assert!(
                call(bad, serde_json::json!([])).is_err(),
                "`{bad}` should be rejected"
            );
        }
    }

    #[test]
    fn a_step_source_accepts_either_a_script_or_a_call() {
        let script: LuaSource = serde_json::from_value(serde_json::json!({ "script": "return 1" }))
            .expect("expected a script source");
        let called: LuaSource = serde_json::from_value(serde_json::json!({ "call": "lights.all" }))
            .expect("expected a call source");

        assert_eq!(script.summary(), "return 1");
        assert_eq!(called.summary(), "call lights.all");
    }
}
