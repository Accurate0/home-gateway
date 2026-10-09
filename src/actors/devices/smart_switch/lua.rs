use mlua::{ExternalError, ExternalResult};

use crate::actors::devices::light::LightHandler;
use crate::actors::devices::light::command::light_message;
use crate::actors::devices::light::query::query_light_on;
use crate::actors::system::rpc;
use crate::lua::{Json, LuaCallContext, lua_module};
use crate::workflows::definition::{LightState, SwitchState};

pub struct SwitchLua;

#[lua_module(namespace = "switch")]
impl SwitchLua {
    #[lua(scope = Switch::Write)]
    async fn set(
        cx: &LuaCallContext,
        device: String,
        command: Json<SwitchState>,
    ) -> mlua::Result<()> {
        let state = command.0;
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

        cx.command(Self::SET, format!("{device} -> {state:?}"), || async {
            rpc::cast_factory(LightHandler::NAME, message)
        })
        .await
    }

    #[lua(scope = Switch::Read)]
    async fn is_on(cx: &LuaCallContext, device: String) -> mlua::Result<bool> {
        let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

        cx.query(Self::IS_ON, || async {
            query_light_on(&ieee_addr, cx.timeout()).await
        })
        .await
    }
}
