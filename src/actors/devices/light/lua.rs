use mlua::ExternalResult;

use crate::actors::system::rpc;
use crate::lua::{Json, LuaCallContext, lua_module};
use crate::settings::workflow::LightState;
use crate::tracing_context::inject_current;

use super::command::light_message;
use super::{LightHandler, LightHandlerMessage};

pub struct LightLua;

#[lua_module(namespace = "light")]
impl LightLua {
    #[lua(scope = Light::Write)]
    async fn set(
        cx: &LuaCallContext,
        device: String,
        command: Json<LightState>,
    ) -> mlua::Result<()> {
        let state = command.0;
        let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();
        let message = light_message(ieee_addr, state.clone()).into_lua_err()?;

        cx.command(Self::SET, format!("{device} -> {state:?}"), || async {
            rpc::cast_factory(LightHandler::NAME, message)
        })
        .await
    }

    #[lua(scope = Light::Read)]
    async fn is_on(cx: &LuaCallContext, device: String) -> mlua::Result<bool> {
        let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

        cx.query(Self::IS_ON, || async {
            rpc::query_factory(LightHandler::NAME, cx.timeout(), |reply| {
                LightHandlerMessage::QueryPowerState {
                    ieee_addr,
                    traceparent: inject_current(),
                    reply,
                }
            })
            .await
        })
        .await
    }
}
