use chrono::{TimeDelta, Utc};

use crate::lua::{Json, LuaCallContext, lua_module};

use crate::sun::{self, SunPeriod};
use crate::timedelta_format::parse_signed;

pub struct SunLua;

#[lua_module(namespace = "sun")]
impl SunLua {
    #[lua]
    fn period(cx: &LuaCallContext, offset: Option<String>) -> mlua::Result<Json<SunPeriod>> {
        Ok(Json(Self::current(cx, offset)?))
    }

    #[lua]
    fn is(
        cx: &LuaCallContext,
        period: Json<SunPeriod>,
        offset: Option<String>,
    ) -> mlua::Result<bool> {
        Ok(Self::current(cx, offset)? == period.0)
    }

    fn current(cx: &LuaCallContext, offset: Option<String>) -> mlua::Result<SunPeriod> {
        let offset = match offset {
            Some(offset) => parse_signed(&offset).map_err(mlua::Error::external)?,
            None => TimeDelta::zero(),
        };

        Ok(sun::current_period(
            cx.state.settings.location,
            Utc::now(),
            offset,
        ))
    }
}
