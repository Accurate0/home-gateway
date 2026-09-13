use mlua::{ExternalResult, Lua, Table, Value as LuaValue};
use regex::Regex;

use super::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const PATTERN: LuaParam = LuaParam {
    name: "pattern",
    ty: LuaType::String,
};

const TEXT: LuaParam = LuaParam {
    name: "text",
    ty: LuaType::String,
};

const TEST: LuaFunction = LuaFunction {
    name: "test",
    params: &[PATTERN, TEXT],
    returns: Some(LuaType::Boolean),
    scope: None,
};

const MATCH: LuaFunction = LuaFunction {
    name: "match",
    params: &[PATTERN, TEXT],
    returns: Some(LuaType::Optional(&LuaType::Table)),
    scope: None,
};

const FIND_ALL: LuaFunction = LuaFunction {
    name: "find_all",
    params: &[PATTERN, TEXT],
    returns: Some(LuaType::Array(&LuaType::String)),
    scope: None,
};

const REPLACE: LuaFunction = LuaFunction {
    name: "replace",
    params: &[
        PATTERN,
        TEXT,
        LuaParam {
            name: "replacement",
            ty: LuaType::String,
        },
    ],
    returns: Some(LuaType::String),
    scope: None,
};

const SPLIT: LuaFunction = LuaFunction {
    name: "split",
    params: &[PATTERN, TEXT],
    returns: Some(LuaType::Array(&LuaType::String)),
    scope: None,
};

const FUNCTIONS: &[LuaFunction] = &[TEST, MATCH, FIND_ALL, REPLACE, SPLIT];

pub struct RegexLua;

fn compile(pattern: &str) -> mlua::Result<Regex> {
    Regex::new(pattern).into_lua_err()
}

fn captures(lua: &Lua, regex: &Regex, text: &str) -> mlua::Result<LuaValue> {
    let Some(found) = regex.captures(text) else {
        return Ok(LuaValue::Nil);
    };

    let result = lua.create_table()?;

    for (index, group) in found.iter().enumerate() {
        if let Some(group) = group {
            result.set(index + 1, group.as_str())?;
        }
    }

    for name in regex.capture_names().flatten() {
        if let Some(group) = found.name(name) {
            result.set(name, group.as_str())?;
        }
    }

    Ok(LuaValue::Table(result))
}

fn strings<'a>(lua: &Lua, items: impl Iterator<Item = &'a str>) -> mlua::Result<Table> {
    let result = lua.create_table()?;

    for item in items {
        result.push(item)?;
    }

    Ok(result)
}

impl LuaModule for RegexLua {
    fn namespace(&self) -> &'static str {
        "re"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        cx.expose(table, &TEST, || {
            lua.create_function(|_, (pattern, text): (String, String)| {
                Ok(compile(&pattern)?.is_match(&text))
            })
        })?;

        cx.expose(table, &MATCH, || {
            lua.create_function(|lua, (pattern, text): (String, String)| {
                captures(lua, &compile(&pattern)?, &text)
            })
        })?;

        cx.expose(table, &FIND_ALL, || {
            lua.create_function(|lua, (pattern, text): (String, String)| {
                let regex = compile(&pattern)?;

                strings(lua, regex.find_iter(&text).map(|found| found.as_str()))
            })
        })?;

        cx.expose(table, &REPLACE, || {
            lua.create_function(
                |_, (pattern, text, replacement): (String, String, String)| {
                    Ok(compile(&pattern)?
                        .replace_all(&text, replacement.as_str())
                        .into_owned())
                },
            )
        })?;

        cx.expose(table, &SPLIT, || {
            lua.create_function(|lua, (pattern, text): (String, String)| {
                let regex = compile(&pattern)?;

                strings(lua, regex.split(&text))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use mlua::{Lua, Table, Value as LuaValue};

    use super::{captures, compile, strings};

    #[test]
    fn captures_expose_positional_and_named_groups() {
        let lua = Lua::new();
        let regex = compile(r"(?<hour>\d{2}):(\d{2})").expect("expected a valid pattern");

        let LuaValue::Table(found) =
            captures(&lua, &regex, "leaves at 07:45").expect("expected captures")
        else {
            panic!("expected a capture table");
        };

        assert_eq!(found.get::<String>(1).expect("whole match"), "07:45");
        assert_eq!(found.get::<String>(2).expect("hour group"), "07");
        assert_eq!(found.get::<String>(3).expect("minute group"), "45");
        assert_eq!(found.get::<String>("hour").expect("named group"), "07");
    }

    #[test]
    fn no_match_is_nil() {
        let lua = Lua::new();
        let regex = compile(r"\d+").expect("expected a valid pattern");

        let found = captures(&lua, &regex, "no digits").expect("expected a result");

        assert!(found.is_nil());
    }

    #[test]
    fn an_invalid_pattern_is_an_error() {
        assert!(compile("(unclosed").is_err());
    }

    #[test]
    fn strings_become_a_sequence() {
        let lua = Lua::new();
        let regex = compile(r",\s*").expect("expected a valid pattern");

        let parts: Table = strings(&lua, regex.split("a, b,c")).expect("expected a table");
        let parts: Vec<String> = parts
            .sequence_values::<String>()
            .collect::<mlua::Result<_>>()
            .expect("expected strings");

        assert_eq!(parts, vec!["a", "b", "c"]);
    }
}
