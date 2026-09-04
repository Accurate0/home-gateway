use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::woolworths_object::WoolworthsObject;
use async_graphql::Object;

#[derive(Default)]
pub struct WoolworthsQuery;

#[Object]
impl WoolworthsQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Woolworths, Action::Read)))]
    async fn woolworths(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<WoolworthsObject> {
        Ok(WoolworthsObject {})
    }
}
