use chrono::DateTime;
use mlua::{ExternalError, Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const INTERVAL: LuaClass = LuaClass {
    name: "EnergyInterval",
    fields: &[
        LuaField {
            name: "used",
            ty: LuaType::Number,
        },
        LuaField {
            name: "exported",
            ty: LuaType::Number,
        },
        LuaField {
            name: "at",
            ty: LuaType::Integer,
        },
    ],
};

const SINCE: LuaFunction = LuaFunction {
    name: "since",
    params: &[LuaParam {
        name: "epoch",
        ty: LuaType::Integer,
    }],
    returns: Some(LuaType::Array(&LuaType::Class(&INTERVAL))),
    scope: Some(Scope::new(Resource::Energy, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[SINCE];

pub struct EnergyLua;

impl LuaModule for EnergyLua {
    fn namespace(&self) -> &'static str {
        "energy"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let since_cx = cx.clone();
        cx.expose(table, &SINCE, || {
            lua.create_async_function(move |lua, epoch: i64| {
                let cx = since_cx.clone();

                async move {
                    let since = DateTime::from_timestamp(epoch, 0)
                        .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

                    let rows = cx
                        .query("energy.since", || async {
                            cx.state.repos.energy().history_since(since).await
                        })
                        .await?;

                    let result = lua.create_table()?;

                    for row in rows {
                        let entry = lua.create_table()?;

                        entry.set("used", row.energy_used)?;
                        entry.set("exported", row.solar_exported)?;
                        entry.set("at", row.time.timestamp())?;

                        result.push(entry)?;
                    }

                    Ok(result)
                }
            })
        })
    }
}
