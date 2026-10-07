use mlua::{ExternalError, ExternalResult, Lua, LuaSerdeExt, Value as LuaValue};
use serde_json::Value;

use super::{LuaCallContext, lua_module};

pub struct StateLua;

fn scoped(key: &str) -> String {
    format!("lua:{key}")
}

async fn read(cx: &LuaCallContext, key: &str) -> mlua::Result<Option<Value>> {
    let raw = cx
        .query(StateLua::GET, || async {
            cx.state.repos.workflow().state_value(key).await
        })
        .await?;

    raw.map(|raw| serde_json::from_str(&raw).into_lua_err())
        .transpose()
}

#[lua_module(namespace = "state")]
impl StateLua {
    #[lua(scope = Workflow::Read)]
    async fn get(cx: &LuaCallContext, lua: &Lua, key: String) -> mlua::Result<LuaValue> {
        match read(cx, &scoped(&key)).await? {
            Some(value) => lua.to_value(&value),
            None => Ok(LuaValue::Nil),
        }
    }

    #[lua(scope = Workflow::Write)]
    async fn set(cx: &LuaCallContext, lua: &Lua, key: String, value: LuaValue) -> mlua::Result<()> {
        let value: Value = lua.from_value(value)?;
        let encoded = serde_json::to_string(&value).into_lua_err()?;
        let key = scoped(&key);

        cx.command(Self::SET, format!("{key} = {encoded}"), || async {
            cx.state
                .repos
                .workflow()
                .set_state_value(&key, &encoded)
                .await
        })
        .await
    }

    #[lua(scope = Workflow::Write)]
    async fn clear(cx: &LuaCallContext, key: String) -> mlua::Result<()> {
        let key = scoped(&key);

        cx.command(Self::CLEAR, &key, || async {
            cx.state.repos.workflow().clear_state_value(&key).await
        })
        .await
    }

    #[lua(scope = Workflow::Write)]
    async fn incr(cx: &LuaCallContext, key: String, by: Option<i64>) -> mlua::Result<i64> {
        let key = scoped(&key);

        let current = match read(cx, &key).await? {
            Some(value) => value
                .as_i64()
                .ok_or_else(|| format!("state `{key}` is not an integer").into_lua_err())?,
            None => 0,
        };

        let next = current + by.unwrap_or(1);
        let encoded = next.to_string();

        cx.command(Self::INCR, format!("{key} = {encoded}"), || async {
            cx.state
                .repos
                .workflow()
                .set_state_value(&key, &encoded)
                .await
        })
        .await?;

        Ok(next)
    }
}
