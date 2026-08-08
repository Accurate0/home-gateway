use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::jellyfin_object::JellyfinObject;
use async_graphql::Object;

#[derive(Default)]
pub struct JellyfinQuery;

#[Object]
impl JellyfinQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_JELLYFIN_READ))]
    async fn jellyfin(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<JellyfinObject> {
        Ok(JellyfinObject {})
    }
}
