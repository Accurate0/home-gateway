use chrono::DateTime;
use mlua::ExternalResult;

use crate::lua::{LuaCallContext, lua_module};

use super::AlarmActor;

pub struct AlarmLua;

#[lua_module(namespace = "alarm")]
impl AlarmLua {
    #[lua(scope = Alarm::Read)]
    async fn next(cx: &LuaCallContext) -> mlua::Result<Option<i64>> {
        let raw = cx
            .query(Self::NEXT, || async {
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
}
