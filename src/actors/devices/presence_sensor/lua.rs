use super::query::{query_presence, query_presence_seen_within};
use crate::lua::{LuaCallContext, lua_module};
use crate::timedelta_format::parse_datetime_str_with_ms;

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

    #[lua(scope = Presence::Read)]
    async fn seen_within(
        cx: &LuaCallContext,
        device: String,
        window: String,
    ) -> mlua::Result<bool> {
        let sensor = cx.state.devices.address_or_self(&device).to_owned();
        let window = parse_datetime_str_with_ms(&window).map_err(mlua::Error::external)?;

        cx.query(Self::SEEN_WITHIN, || async {
            query_presence_seen_within(&sensor, window, cx.timeout()).await
        })
        .await
    }
}
