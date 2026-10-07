use mlua::{ExternalError, Lua, LuaSerdeExt, Table, Value as LuaValue};
use serde_json::{Map, Value};

use crate::lua::{LuaCallContext, lua_module};

use super::HomeAssistant;

pub struct HomeAssistantLua;

fn client(cx: &LuaCallContext) -> mlua::Result<HomeAssistant> {
    cx.state
        .handles
        .get::<HomeAssistant>()
        .cloned()
        .ok_or_else(|| "home assistant is not configured".into_lua_err())
}

#[lua_module(namespace = "home_assistant")]
impl HomeAssistantLua {
    #[lua(scope = HomeAssistant::Write)]
    async fn call(
        cx: &LuaCallContext,
        lua: &Lua,
        service: String,
        data: Option<Table>,
    ) -> mlua::Result<()> {
        let (domain, name) = service.split_once('.').ok_or_else(|| {
            format!("`{service}` must be written as `domain.service`").into_lua_err()
        })?;

        let data = match data {
            Some(data) => lua.from_value(LuaValue::Table(data))?,
            None => Value::Object(Map::new()),
        };

        let home_assistant = client(cx)?;

        cx.command(Self::CALL, &service, || async {
            home_assistant.call_service(domain, name, data).await
        })
        .await
    }

    #[lua(scope = HomeAssistant::Read)]
    async fn state(cx: &LuaCallContext, lua: &Lua, entity_id: String) -> mlua::Result<LuaValue> {
        let home_assistant = client(cx)?;

        let entity = cx
            .query(Self::STATE, || async {
                home_assistant.get_state(&entity_id).await
            })
            .await?;

        lua.to_value(&entity)
    }
}
