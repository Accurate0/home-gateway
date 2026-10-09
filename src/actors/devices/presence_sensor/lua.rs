use super::query::query_presence;
use crate::lua::{LuaCallContext, lua_module};

pub struct PresenceLua;

#[lua_module(namespace = "presence")]
impl PresenceLua {
    #[lua(scope = Presence::Read)]
    async fn at(cx: &LuaCallContext, device: String) -> mlua::Result<bool> {
        let sensor = cx.state.devices.address_or_self(&device).to_owned();

        cx.query(Self::AT, || async {
            query_presence(&sensor, cx.timeout()).await
        })
        .await
    }
}
