use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::woolworths_object::WoolworthsObject;
use async_graphql::Object;

#[derive(Default)]
pub struct WoolworthsQuery;

#[Object]
impl WoolworthsQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_WOOLWORTHS_READ))]
    async fn woolworths(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<WoolworthsObject> {
        Ok(WoolworthsObject {})
    }
}
