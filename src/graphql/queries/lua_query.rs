use async_graphql::{Context, Object};

use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::lua_api_object::LuaApiObject;
use crate::integrations::home_assistant::HomeAssistant;
use crate::startup::lua::{api_description, definitions};
use crate::state::AppState;

#[derive(Default)]
pub struct LuaQuery;

#[Object]
impl LuaQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Lua, Action::Read)))]
    async fn lua_api(&self, ctx: &Context<'_>) -> async_graphql::Result<LuaApiObject> {
        let state = ctx.data::<AppState>()?;
        let home_assistant = state.handles.contains::<HomeAssistant>();

        Ok(LuaApiObject::new(
            api_description(home_assistant),
            definitions(home_assistant),
        ))
    }
}
