use mlua::{ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::actors::system::rpc;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType, schema};
use crate::settings::workflow::LightState;

use super::command::light_message;
use super::{LightHandler, LightHandlerMessage};

const SET: LuaFunction = LuaFunction {
    name: "set",
    params: &[
        LuaParam {
            name: "device",
            ty: LuaType::String,
        },
        LuaParam {
            name: "command",
            ty: LuaType::Schema(schema::<LightState>),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Light, Action::Write)),
};

const IS_ON: LuaFunction = LuaFunction {
    name: "is_on",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Light, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[SET, IS_ON];

pub struct LightLua;

impl LuaModule for LightLua {
    fn namespace(&self) -> &'static str {
        "light"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let set_cx = cx.clone();
        cx.expose(table, &SET, || {
            lua.create_async_function(move |lua, (device, state): (String, LuaValue)| {
                let cx = set_cx.clone();

                async move {
                    let state: LightState = lua.from_value(state)?;
                    let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();
                    let message = light_message(ieee_addr, state.clone()).into_lua_err()?;

                    cx.command("light.set", format!("{device} -> {state:?}"), || async {
                        rpc::cast_factory(LightHandler::NAME, message)
                    })
                    .await
                }
            })
        })?;

        let is_on_cx = cx.clone();
        cx.expose(table, &IS_ON, || {
            lua.create_async_function(move |_, device: String| {
                let cx = is_on_cx.clone();

                async move {
                    let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

                    cx.query("light.is_on", || async {
                        rpc::query_factory(LightHandler::NAME, cx.timeout(), |reply| {
                            LightHandlerMessage::QueryPowerState { ieee_addr, reply }
                        })
                        .await
                    })
                    .await
                }
            })
        })
    }
}
