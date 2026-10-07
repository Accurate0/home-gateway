use mlua::{ExternalResult, Lua, Table};
use regex::Regex;

use super::lua_module;

pub struct RegexLua;

fn compile(pattern: &str) -> mlua::Result<Regex> {
    Regex::new(pattern).into_lua_err()
}

fn captures(lua: &Lua, regex: &Regex, text: &str) -> mlua::Result<Option<Table>> {
    let Some(found) = regex.captures(text) else {
        return Ok(None);
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

    Ok(Some(result))
}

#[lua_module(namespace = "re")]
impl RegexLua {
    #[lua]
    fn test(pattern: String, text: String) -> mlua::Result<bool> {
        Ok(compile(&pattern)?.is_match(&text))
    }

    #[lua]
    fn r#match(lua: &Lua, pattern: String, text: String) -> mlua::Result<Option<Table>> {
        captures(lua, &compile(&pattern)?, &text)
    }

    #[lua]
    fn find_all(pattern: String, text: String) -> mlua::Result<Vec<String>> {
        let regex = compile(&pattern)?;

        Ok(regex
            .find_iter(&text)
            .map(|found| found.as_str().to_owned())
            .collect())
    }

    #[lua]
    fn replace(pattern: String, text: String, replacement: String) -> mlua::Result<String> {
        Ok(compile(&pattern)?
            .replace_all(&text, replacement.as_str())
            .into_owned())
    }

    #[lua]
    fn split(pattern: String, text: String) -> mlua::Result<Vec<String>> {
        let regex = compile(&pattern)?;

        Ok(regex.split(&text).map(str::to_owned).collect())
    }
}

#[cfg(test)]
mod tests {
    use mlua::Lua;

    use super::{RegexLua, captures, compile};

    #[test]
    fn captures_expose_positional_and_named_groups() {
        let lua = Lua::new();
        let regex = compile(r"(?<hour>\d{2}):(\d{2})").expect("expected a valid pattern");

        let found = captures(&lua, &regex, "leaves at 07:45")
            .expect("expected captures")
            .expect("expected a capture table");

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

        assert!(found.is_none());
    }

    #[test]
    fn an_invalid_pattern_is_an_error() {
        assert!(compile("(unclosed").is_err());
    }

    #[test]
    fn split_returns_the_parts_in_order() {
        let parts =
            RegexLua::split(r",\s*".to_owned(), "a, b,c".to_owned()).expect("expected the parts");

        assert_eq!(parts, vec!["a", "b", "c"]);
    }
}
