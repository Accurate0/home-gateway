use mlua::{ExternalError, ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::actors::devices::light::LightHandler;
use crate::actors::devices::light::command::light_message;
use crate::actors::system::rpc;
use crate::actors::workflows::conditions::query_light_on;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType, schema};
use crate::settings::workflow::{LightState, SwitchState};

const SET: LuaFunction = LuaFunction {
    name: "set",
    params: &[
        LuaParam {
            name: "device",
            ty: LuaType::String,
        },
        LuaParam {
            name: "command",
            ty: LuaType::Schema(schema::<SwitchState>),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Switch, Action::Write)),
};

const IS_ON: LuaFunction = LuaFunction {
    name: "is_on",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Switch, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[SET, IS_ON];

pub struct SwitchLua;

impl LuaModule for SwitchLua {
    fn namespace(&self) -> &'static str {
        "switch"
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
                    let state: SwitchState = lua.from_value(state)?;
                    let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

                    if cx.state.devices.light(&ieee_addr).is_none() {
                        return Err(format!(
                            "switch `{device}` has no control path: only a switch declared `as: light` can be driven"
                        )
                        .into_lua_err());
                    }

                    let light_state = match state {
                        SwitchState::On => LightState::On,
                        SwitchState::Off => LightState::Off,
                        SwitchState::Toggle => LightState::Toggle,
                    };

                    let message = light_message(ieee_addr, light_state).into_lua_err()?;

                    cx.command("switch.set", format!("{device} -> {state:?}"), || async {
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

                    cx.query("switch.is_on", || async {
                        query_light_on(&ieee_addr, cx.timeout()).await
                    })
                    .await
                }
            })
        })
    }
}
