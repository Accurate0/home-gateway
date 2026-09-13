use mlua::{ExternalError, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

use super::HomeAssistant;

const CALL: LuaFunction = LuaFunction {
    name: "call",
    params: &[
        LuaParam {
            name: "service",
            ty: LuaType::String,
        },
        LuaParam {
            name: "data",
            ty: LuaType::Optional(&LuaType::Table),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::HomeAssistant, Action::Write)),
};

const STATE: LuaFunction = LuaFunction {
    name: "state",
    params: &[LuaParam {
        name: "entity_id",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Any),
    scope: Some(Scope::new(Resource::HomeAssistant, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[CALL, STATE];

pub struct HomeAssistantLua;

impl LuaModule for HomeAssistantLua {
    fn namespace(&self) -> &'static str {
        "home_assistant"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let call_cx = cx.clone();
        cx.expose(table, &CALL, || {
            lua.create_async_function(move |lua, (service, data): (String, Option<LuaValue>)| {
                let cx = call_cx.clone();

                async move {
                    let (domain, name) = service.split_once('.').ok_or_else(|| {
                        format!("`{service}` must be written as `domain.service`").into_lua_err()
                    })?;

                    let data = match data {
                        Some(data) => lua.from_value(data)?,
                        None => serde_json::Value::Object(serde_json::Map::new()),
                    };

                    let home_assistant = cx
                        .state
                        .handles
                        .get::<HomeAssistant>()
                        .ok_or_else(|| "home assistant is not configured".into_lua_err())?
                        .clone();

                    let domain = domain.to_owned();
                    let name = name.to_owned();

                    cx.command("home_assistant.call", &service, || async {
                        home_assistant.call_service(&domain, &name, data).await
                    })
                    .await
                }
            })
        })?;

        let state_cx = cx.clone();
        cx.expose(table, &STATE, || {
            lua.create_async_function(move |lua, entity_id: String| {
                let cx = state_cx.clone();

                async move {
                    let home_assistant = cx
                        .state
                        .handles
                        .get::<HomeAssistant>()
                        .ok_or_else(|| "home assistant is not configured".into_lua_err())?
                        .clone();

                    let entity = cx
                        .query("home_assistant.state", || async {
                            home_assistant.get_state(&entity_id).await
                        })
                        .await?;

                    lua.to_value(&entity)
                }
            })
        })
    }
}
