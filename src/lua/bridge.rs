use std::collections::BTreeMap;

use mlua::{Lua, Table, Value as LuaValue};

use crate::variables::{Node, Value, VarType, Vars};

use super::LuaError;

pub fn install_vars(lua: &Lua, vars: &Vars) -> mlua::Result<()> {
    let globals = lua.globals();

    for (namespace, node) in vars.iter() {
        globals.set(namespace.as_str(), node_to_lua(lua, node)?)?;
    }

    Ok(())
}

pub fn node_to_lua(lua: &Lua, node: &Node) -> mlua::Result<LuaValue> {
    match node {
        Node::Value(None) => Ok(LuaValue::Nil),
        Node::Value(Some(value)) => value_to_lua(lua, value),
        Node::Object(fields) => {
            let table = lua.create_table()?;

            for (key, field) in fields {
                table.set(key.as_str(), node_to_lua(lua, field)?)?;
            }

            Ok(LuaValue::Table(table))
        }
    }
}

pub fn value_to_lua(lua: &Lua, value: &Value) -> mlua::Result<LuaValue> {
    let converted = match value {
        Value::String(value) => LuaValue::String(lua.create_string(value)?),
        Value::Int(value) => LuaValue::Integer(*value),
        Value::Float(value) => LuaValue::Number(*value),
        Value::Bool(value) => LuaValue::Boolean(*value),
    };

    Ok(converted)
}

pub fn lua_to_value(value: &LuaValue) -> Option<Value> {
    match value {
        LuaValue::String(text) => Some(Value::String(text.to_string_lossy())),
        LuaValue::Integer(number) => Some(Value::Int(*number)),
        LuaValue::Number(number) => Some(Value::Float(*number)),
        LuaValue::Boolean(flag) => Some(Value::Bool(*flag)),
        _ => None,
    }
}

pub fn returned_node(
    table: &Table,
    declared: &BTreeMap<String, VarType>,
) -> Result<Node, LuaError> {
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair.map_err(LuaError::from_mlua)?;

        let Some(key) = key.as_string().map(|key| key.to_string_lossy()) else {
            continue;
        };

        if !declared.contains_key(&key) {
            return Err(LuaError::UndeclaredReturn {
                key,
                declared: declared.keys().cloned().collect::<Vec<_>>().join(", "),
            });
        }
    }

    let mut node = Node::empty();

    for (key, ty) in declared {
        let raw: LuaValue = table.get(key.as_str()).map_err(LuaError::from_mlua)?;

        if raw.is_nil() {
            return Err(LuaError::MissingReturn { key: key.clone() });
        }

        let value = lua_to_value(&raw)
            .and_then(|value| value.coerce(*ty))
            .ok_or_else(|| LuaError::ReturnValue {
                key: key.clone(),
                got: raw.type_name().to_owned(),
                expected: *ty,
            })?;

        node.insert(key.clone(), Node::Value(Some(value)));
    }

    Ok(node)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use mlua::{Lua, Value as LuaValue};

    use crate::variables::{Node, Value, VarType, Vars};

    use super::{install_vars, returned_node};

    #[test]
    fn vars_are_installed_as_lua_globals() {
        let lua = Lua::new();

        let mut event = Node::empty();
        event.insert("power", Node::Value(Some(Value::Float(4200.5))));
        event.insert(
            "device",
            Node::Value(Some(Value::String("lamp".to_owned()))),
        );
        event.insert("missing", Node::Value(None));

        let vars = Vars::default().with("event", event);
        install_vars(&lua, &vars).expect("expected the vars to install");

        let power: f64 = lua.load("return event.power").eval().expect("power");
        let device: String = lua.load("return event.device").eval().expect("device");
        let missing: LuaValue = lua.load("return event.missing").eval().expect("missing");

        assert_eq!(power, 4200.5);
        assert_eq!(device, "lamp");
        assert!(missing.is_nil(), "an absent value should arrive as nil");
    }

    #[test]
    fn a_returned_table_is_coerced_to_the_declared_types() {
        let lua = Lua::new();
        let table = lua
            .load("return { pct = 84, label = \"peak\", ratio = 2 }")
            .eval()
            .expect("expected a table");

        let declared = BTreeMap::from([
            ("pct".to_owned(), VarType::Int),
            ("label".to_owned(), VarType::String),
            ("ratio".to_owned(), VarType::Float),
        ]);

        let node = returned_node(&table, &declared).expect("expected the table to coerce");

        let Node::Object(fields) = node else {
            panic!("expected an object node");
        };

        assert_eq!(fields["pct"], Node::Value(Some(Value::Int(84))));
        assert_eq!(
            fields["label"],
            Node::Value(Some(Value::String("peak".to_owned())))
        );
        assert_eq!(fields["ratio"], Node::Value(Some(Value::Float(2.0))));
    }

    #[test]
    fn a_returned_table_rejects_undeclared_and_missing_keys() {
        let lua = Lua::new();
        let declared = BTreeMap::from([("pct".to_owned(), VarType::Int)]);

        let extra = lua
            .load("return { pct = 1, other = 2 }")
            .eval()
            .expect("expected a table");
        let absent = lua.load("return { }").eval().expect("expected a table");

        assert!(matches!(
            returned_node(&extra, &declared),
            Err(crate::lua::LuaError::UndeclaredReturn { .. })
        ));
        assert!(matches!(
            returned_node(&absent, &declared),
            Err(crate::lua::LuaError::MissingReturn { .. })
        ));
    }
}
