use chrono::{TimeDelta, Utc};

use crate::lua::{Json, LuaCallContext, lua_module};

use crate::sun::{self, SunPeriod};

pub struct SunLua;

#[lua_module(namespace = "sun")]
impl SunLua {
    #[lua]
    fn period(cx: &LuaCallContext) -> mlua::Result<Json<SunPeriod>> {
        Ok(Json(Self::current(cx)))
    }

    #[lua]
    fn is(cx: &LuaCallContext, period: Json<SunPeriod>) -> mlua::Result<bool> {
        Ok(Self::current(cx) == period.0)
    }

    fn current(cx: &LuaCallContext) -> SunPeriod {
        sun::current_period(cx.state.settings.location, Utc::now(), TimeDelta::zero())
    }
}
