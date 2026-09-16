use mlua::chunk::{Chunk, ChunkMode};
use mlua::{Lua, Value as LuaValue};

pub fn compile(name: &str, source: &str) -> Result<Vec<u8>, String> {
    let lua = Lua::new();

    let expression = lua
        .load(format!("return {source}"))
        .set_name(name)
        .into_function();

    let function = match expression {
        Ok(function) => function,
        Err(_) => lua
            .load(source)
            .set_name(name)
            .into_function()
            .map_err(|error| error.to_string())?,
    };

    Ok(function.dump(false))
}

pub fn load<'lua>(lua: &'lua Lua, name: &str, bytecode: &[u8]) -> mlua::Result<Chunk<'lua>> {
    Ok(lua
        .load(bytecode.to_vec())
        .set_name(name)
        .set_mode(ChunkMode::Binary))
}

pub async fn eval(lua: &Lua, name: &str, bytecode: &[u8]) -> mlua::Result<LuaValue> {
    load(lua, name, bytecode)?.call_async::<LuaValue>(()).await
}

#[cfg(test)]
mod tests {
    use super::{compile, eval};
    use mlua::Value as LuaValue;

    #[tokio::test]
    async fn a_statement_chunk_round_trips_through_bytecode() {
        let bytecode =
            compile("script", "return { a = 1 }").expect("expected the chunk to compile");

        let lua = mlua::Lua::new();
        let value = eval(&lua, "script", &bytecode)
            .await
            .expect("expected the chunk to run");

        let LuaValue::Table(table) = value else {
            panic!("expected a table");
        };

        assert_eq!(table.get::<i64>("a").expect("expected a field"), 1);
    }

    #[tokio::test]
    async fn a_bare_expression_keeps_its_value() {
        let bytecode = compile("script", "1 + 1").expect("expected the chunk to compile");

        let lua = mlua::Lua::new();
        let value = eval(&lua, "script", &bytecode)
            .await
            .expect("expected the chunk to run");

        assert_eq!(value.as_i64(), Some(2));
    }

    #[test]
    fn a_syntax_error_is_reported() {
        let error = compile("script", "return 1 +").expect_err("expected a syntax error");

        assert!(error.contains("syntax"), "unexpected error: {error}");
    }
}
