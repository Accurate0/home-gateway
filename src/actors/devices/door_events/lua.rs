use super::query::query_door_open;
use crate::lua::{LuaCallContext, lua_module};

pub struct DoorLua;

#[lua_module(namespace = "door")]
impl DoorLua {
    #[lua(scope = Door::Read)]
    async fn is_open(cx: &LuaCallContext, device: String) -> mlua::Result<bool> {
        let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

        cx.query(Self::IS_OPEN, || async {
            query_door_open(&ieee_addr, cx.timeout()).await
        })
        .await
    }
}
