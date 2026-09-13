use mlua::{ExternalError, ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};

use super::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const KEY: LuaParam = LuaParam {
    name: "key",
    ty: LuaType::String,
};

const GET: LuaFunction = LuaFunction {
    name: "get",
    params: &[KEY],
    returns: Some(LuaType::Any),
    scope: Some(Scope::new(Resource::Workflow, Action::Read)),
};

const SET: LuaFunction = LuaFunction {
    name: "set",
    params: &[
        KEY,
        LuaParam {
            name: "value",
            ty: LuaType::Any,
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Workflow, Action::Write)),
};

const CLEAR: LuaFunction = LuaFunction {
    name: "clear",
    params: &[KEY],
    returns: None,
    scope: Some(Scope::new(Resource::Workflow, Action::Write)),
};

const INCR: LuaFunction = LuaFunction {
    name: "incr",
    params: &[
        KEY,
        LuaParam {
            name: "by",
            ty: LuaType::Optional(&LuaType::Integer),
        },
    ],
    returns: Some(LuaType::Integer),
    scope: Some(Scope::new(Resource::Workflow, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[GET, SET, CLEAR, INCR];

pub struct StateLua;

fn scoped(key: &str) -> String {
    format!("lua:{key}")
}

async fn read(cx: &LuaCallContext, key: &str) -> mlua::Result<Option<serde_json::Value>> {
    let raw = cx
        .query("state.get", || async {
            cx.state.repos.workflow().state_value(key).await
        })
        .await?;

    raw.map(|raw| serde_json::from_str(&raw).into_lua_err())
        .transpose()
}

impl LuaModule for StateLua {
    fn namespace(&self) -> &'static str {
        "state"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let get_cx = cx.clone();
        cx.expose(table, &GET, || {
            lua.create_async_function(move |lua, key: String| {
                let cx = get_cx.clone();

                async move {
                    match read(&cx, &scoped(&key)).await? {
                        Some(value) => lua.to_value(&value),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })
        })?;

        let set_cx = cx.clone();
        cx.expose(table, &SET, || {
            lua.create_async_function(move |lua, (key, value): (String, LuaValue)| {
                let cx = set_cx.clone();

                async move {
                    let value: serde_json::Value = lua.from_value(value)?;
                    let encoded = serde_json::to_string(&value).into_lua_err()?;
                    let key = scoped(&key);

                    cx.command("state.set", format!("{key} = {encoded}"), || async {
                        cx.state
                            .repos
                            .workflow()
                            .set_state_value(&key, &encoded)
                            .await
                    })
                    .await
                }
            })
        })?;

        let clear_cx = cx.clone();
        cx.expose(table, &CLEAR, || {
            lua.create_async_function(move |_, key: String| {
                let cx = clear_cx.clone();

                async move {
                    let key = scoped(&key);

                    cx.command("state.clear", &key, || async {
                        cx.state.repos.workflow().clear_state_value(&key).await
                    })
                    .await
                }
            })
        })?;

        let incr_cx = cx.clone();
        cx.expose(table, &INCR, || {
            lua.create_async_function(move |_, (key, by): (String, Option<i64>)| {
                let cx = incr_cx.clone();

                async move {
                    let key = scoped(&key);

                    let current = match read(&cx, &key).await? {
                        Some(value) => value.as_i64().ok_or_else(|| {
                            format!("state `{key}` is not an integer").into_lua_err()
                        })?,
                        None => 0,
                    };

                    let next = current + by.unwrap_or(1);
                    let encoded = next.to_string();

                    cx.command("state.incr", format!("{key} = {encoded}"), || async {
                        cx.state
                            .repos
                            .workflow()
                            .set_state_value(&key, &encoded)
                            .await
                    })
                    .await?;

                    Ok(next)
                }
            })
        })
    }
}
