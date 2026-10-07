use crate::lua::{LuaCallContext, lua_module};

use super::types::FuelSite;

pub struct FuelLua;

#[lua_module(namespace = "fuel")]
impl FuelLua {
    #[lua(scope = FuelWatch::Read)]
    async fn cheapest(
        cx: &LuaCallContext,
        postcode: Option<i32>,
        limit: Option<i64>,
    ) -> mlua::Result<Vec<FuelSite>> {
        let postcode = postcode.unwrap_or(cx.state.settings.integrations.fuelwatch.postcode);

        cx.query(Self::CHEAPEST, || async {
            cx.state
                .repos
                .fuelwatch()
                .sites_for_postcode(postcode, limit)
                .await
        })
        .await
    }
}
