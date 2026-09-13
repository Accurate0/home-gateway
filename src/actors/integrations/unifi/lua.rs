use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::db::UnifiState;
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const CLIENT: LuaClass = LuaClass {
    name: "UnifiClient",
    fields: &[
        LuaField {
            name: "name",
            ty: LuaType::String,
        },
        LuaField {
            name: "connected",
            ty: LuaType::Boolean,
        },
        LuaField {
            name: "since",
            ty: LuaType::Integer,
        },
    ],
};

const HOME: LuaFunction = LuaFunction {
    name: "home",
    params: &[LuaParam {
        name: "client",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Unifi, Action::Read)),
};

const CLIENTS: LuaFunction = LuaFunction {
    name: "clients",
    params: &[],
    returns: Some(LuaType::Array(&LuaType::Class(&CLIENT))),
    scope: Some(Scope::new(Resource::Unifi, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[HOME, CLIENTS];

pub struct UnifiLua;

impl LuaModule for UnifiLua {
    fn namespace(&self) -> &'static str {
        "unifi"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let home_cx = cx.clone();
        cx.expose(table, &HOME, || {
            lua.create_async_function(move |_, client: String| {
                let cx = home_cx.clone();

                async move {
                    let rows = cx
                        .query("unifi.home", || async {
                            cx.state.repos.unifi().latest_states().await
                        })
                        .await?;

                    Ok(rows
                        .iter()
                        .any(|row| row.name == client && row.state == UnifiState::Connected))
                }
            })
        })?;

        let clients_cx = cx.clone();
        cx.expose(table, &CLIENTS, || {
            lua.create_async_function(move |lua, ()| {
                let cx = clients_cx.clone();

                async move {
                    let rows = cx
                        .query("unifi.clients", || async {
                            cx.state.repos.unifi().latest_states().await
                        })
                        .await?;

                    let result = lua.create_table()?;

                    for row in rows {
                        let entry = lua.create_table()?;

                        entry.set("name", row.name)?;
                        entry.set("connected", row.state == UnifiState::Connected)?;
                        entry.set("since", row.time.timestamp())?;

                        result.push(entry)?;
                    }

                    Ok(result)
                }
            })
        })
    }
}
