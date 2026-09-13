use chrono::DateTime;
use mlua::{ExternalResult, Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaType};

use super::AlarmActor;

const NEXT: LuaFunction = LuaFunction {
    name: "next",
    params: &[],
    returns: Some(LuaType::Optional(&LuaType::Integer)),
    scope: Some(Scope::new(Resource::Alarm, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[NEXT];

pub struct AlarmLua;

impl LuaModule for AlarmLua {
    fn namespace(&self) -> &'static str {
        "alarm"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let next_cx = cx.clone();
        cx.expose(table, &NEXT, || {
            lua.create_async_function(move |_, ()| {
                let cx = next_cx.clone();

                async move {
                    let raw = cx
                        .query("alarm.next", || async {
                            cx.state
                                .repos
                                .workflow()
                                .state_value(AlarmActor::ALARM_STATE_KEY)
                                .await
                        })
                        .await?;

                    match raw {
                        Some(raw) => Ok(Some(
                            DateTime::parse_from_rfc3339(&raw)
                                .into_lua_err()?
                                .timestamp(),
                        )),
                        None => Ok(None),
                    }
                }
            })
        })
    }
}
