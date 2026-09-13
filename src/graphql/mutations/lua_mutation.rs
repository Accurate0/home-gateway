use std::collections::BTreeMap;

use async_graphql::{Context, Json, Object};

use crate::auth::AuthContext;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::lua::{Script, execute};
use crate::state::AppState;

#[derive(Default)]
pub struct LuaMutation;

#[Object]
impl LuaMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Lua, Action::Write)))]
    async fn execute_lua(
        &self,
        ctx: &Context<'_>,
        script: String,
        vars: Option<Json<BTreeMap<String, serde_json::Value>>>,
        dry_run: Option<bool>,
    ) -> async_graphql::Result<Json<serde_json::Value>> {
        let state = ctx.data::<AppState>()?;
        let auth = ctx.data::<AuthContext>()?.clone();
        let script = Script::parse(&script).map_err(async_graphql::Error::new)?;

        let result = execute::execute(
            state,
            auth,
            &script,
            vars.map(|vars| vars.0).unwrap_or_default(),
            dry_run.unwrap_or(false),
        )
        .await
        .map_err(|error| async_graphql::Error::new(error.to_string()))?;

        Ok(Json(result))
    }
}
