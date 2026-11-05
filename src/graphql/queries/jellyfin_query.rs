use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::jellyfin_object::JellyfinObject;
use async_graphql::Object;

#[derive(Default)]
pub struct JellyfinQuery;

#[home_gateway_macros::prefixed("home")]
#[Object]
impl JellyfinQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Jellyfin, Action::Read)))]
    async fn jellyfin(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<JellyfinObject> {
        Ok(JellyfinObject)
    }
}
