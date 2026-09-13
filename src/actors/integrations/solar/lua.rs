use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::integrations::solar;
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaType};

const AVERAGES: LuaClass = LuaClass {
    name: "SolarAverages",
    fields: &[
        LuaField {
            name: "last_15_mins",
            ty: LuaType::Number,
        },
        LuaField {
            name: "last_1_hour",
            ty: LuaType::Number,
        },
        LuaField {
            name: "last_3_hours",
            ty: LuaType::Number,
        },
    ],
};

const CURRENT: LuaFunction = LuaFunction {
    name: "current",
    params: &[],
    returns: Some(LuaType::Optional(&LuaType::Number)),
    scope: Some(Scope::new(Resource::Solar, Action::Read)),
};

const AVERAGES_FN: LuaFunction = LuaFunction {
    name: "averages",
    params: &[],
    returns: Some(LuaType::Class(&AVERAGES)),
    scope: Some(Scope::new(Resource::Solar, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[CURRENT, AVERAGES_FN];

pub struct SolarLua;

impl LuaModule for SolarLua {
    fn namespace(&self) -> &'static str {
        "solar"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let current_cx = cx.clone();
        cx.expose(table, &CURRENT, || {
            lua.create_async_function(move |_, ()| {
                let cx = current_cx.clone();

                async move {
                    cx.query("solar.current", || async {
                        solar::queries::current_wh(&cx.state.db).await
                    })
                    .await
                }
            })
        })?;

        let averages_cx = cx.clone();
        cx.expose(table, &AVERAGES_FN, || {
            lua.create_async_function(move |lua, ()| {
                let cx = averages_cx.clone();

                async move {
                    let statistics = cx
                        .query("solar.averages", || async {
                            solar::queries::statistics(&cx.state.db).await
                        })
                        .await?;

                    let table = lua.create_table()?;
                    table.set("last_15_mins", statistics.averages.last_15_mins)?;
                    table.set("last_1_hour", statistics.averages.last_1_hour)?;
                    table.set("last_3_hours", statistics.averages.last_3_hours)?;

                    Ok(table)
                }
            })
        })
    }
}
