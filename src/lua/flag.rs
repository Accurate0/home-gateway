use mlua::{Lua, Table, Value as LuaValue};
use open_feature::{EvaluationContext, StructValue, Value as FlagValue};

use crate::auth::scope::{Action, Resource, Scope};

use super::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const NAME: LuaParam = LuaParam {
    name: "name",
    ty: LuaType::String,
};

const ENABLED: LuaFunction = LuaFunction {
    name: "enabled",
    params: &[
        NAME,
        LuaParam {
            name: "default",
            ty: LuaType::Boolean,
        },
    ],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::FeatureFlag, Action::Read)),
};

const GET: LuaFunction = LuaFunction {
    name: "get",
    params: &[NAME],
    returns: Some(LuaType::Optional(&LuaType::Map(&LuaType::Any))),
    scope: Some(Scope::new(Resource::FeatureFlag, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[ENABLED, GET];

pub struct FlagLua;

fn flag_to_lua(lua: &Lua, value: &FlagValue) -> mlua::Result<LuaValue> {
    let converted = match value {
        FlagValue::Bool(flag) => LuaValue::Boolean(*flag),
        FlagValue::Int(number) => LuaValue::Integer(*number),
        FlagValue::Float(number) => LuaValue::Number(*number),
        FlagValue::String(text) => LuaValue::String(lua.create_string(text)?),
        FlagValue::Array(items) => {
            let table = lua.create_table()?;

            for item in items {
                table.push(flag_to_lua(lua, item)?)?;
            }

            LuaValue::Table(table)
        }
        FlagValue::Struct(fields) => LuaValue::Table(struct_to_lua(lua, fields)?),
    };

    Ok(converted)
}

fn struct_to_lua(lua: &Lua, value: &StructValue) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    for (key, field) in &value.fields {
        table.set(key.as_str(), flag_to_lua(lua, field)?)?;
    }

    Ok(table)
}

impl LuaModule for FlagLua {
    fn namespace(&self) -> &'static str {
        "flag"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let enabled_cx = cx.clone();
        cx.expose(table, &ENABLED, || {
            lua.create_async_function(move |_, (name, default): (String, bool)| {
                let cx = enabled_cx.clone();

                async move {
                    let enabled = cx
                        .state
                        .feature_flag_client
                        .is_feature_enabled(&name, default, EvaluationContext::default())
                        .await;

                    tracing::debug!("[{}] lua flag {name} evaluated to {enabled}", cx.event_id);

                    Ok(enabled)
                }
            })
        })?;

        let get_cx = cx.clone();
        cx.expose(table, &GET, || {
            lua.create_async_function(move |lua, name: String| {
                let cx = get_cx.clone();

                async move {
                    let value = cx
                        .state
                        .feature_flag_client
                        .get_struct(&name, EvaluationContext::default())
                        .await;

                    match value {
                        Ok(value) => Ok(LuaValue::Table(struct_to_lua(&lua, &value)?)),
                        Err(e) => {
                            tracing::warn!(
                                "[{}] lua flag {name} could not be evaluated: {e:?}",
                                cx.event_id
                            );

                            Ok(LuaValue::Nil)
                        }
                    }
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mlua::{Lua, Table};
    use open_feature::{StructValue, Value as FlagValue};

    use super::struct_to_lua;

    #[test]
    fn a_flag_struct_becomes_a_nested_table() {
        let lua = Lua::new();

        let inner = StructValue {
            fields: HashMap::from([("level".to_owned(), FlagValue::Int(3))]),
        };

        let value = StructValue {
            fields: HashMap::from([
                ("enabled".to_owned(), FlagValue::Bool(true)),
                ("ratio".to_owned(), FlagValue::Float(0.5)),
                ("label".to_owned(), FlagValue::String("beta".to_owned())),
                (
                    "rooms".to_owned(),
                    FlagValue::Array(vec![FlagValue::String("lounge".to_owned())]),
                ),
                ("nested".to_owned(), FlagValue::Struct(inner)),
            ]),
        };

        let table = struct_to_lua(&lua, &value).expect("expected a table");

        assert!(table.get::<bool>("enabled").expect("enabled"));
        assert_eq!(table.get::<f64>("ratio").expect("ratio"), 0.5);
        assert_eq!(table.get::<String>("label").expect("label"), "beta");

        let rooms: Table = table.get("rooms").expect("rooms");
        assert_eq!(rooms.get::<String>(1).expect("first room"), "lounge");

        let nested: Table = table.get("nested").expect("nested");
        assert_eq!(nested.get::<i64>("level").expect("level"), 3);
    }
}
