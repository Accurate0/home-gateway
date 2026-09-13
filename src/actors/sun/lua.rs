use chrono::{TimeDelta, Utc};
use mlua::{Lua, Table};

use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType, schema};

use super::calc::{self, SunPeriod};

const PERIOD: LuaFunction = LuaFunction {
    name: "period",
    params: &[],
    returns: Some(LuaType::Schema(schema::<SunPeriod>)),
    scope: None,
};

const IS: LuaFunction = LuaFunction {
    name: "is",
    params: &[LuaParam {
        name: "period",
        ty: LuaType::Schema(schema::<SunPeriod>),
    }],
    returns: Some(LuaType::Boolean),
    scope: None,
};

const FUNCTIONS: &[LuaFunction] = &[PERIOD, IS];

pub struct SunLua;

impl LuaModule for SunLua {
    fn namespace(&self) -> &'static str {
        "sun"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let location = cx.state.settings.location;

        cx.expose(table, &PERIOD, || {
            lua.create_function(move |_, ()| {
                Ok(calc::current_period(location, Utc::now(), TimeDelta::zero()).to_string())
            })
        })?;

        cx.expose(table, &IS, || {
            lua.create_function(move |_, period: String| {
                Ok(
                    calc::current_period(location, Utc::now(), TimeDelta::zero()).to_string()
                        == period,
                )
            })
        })
    }
}
