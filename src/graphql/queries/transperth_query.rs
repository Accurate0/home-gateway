use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::transperth_object::TransperthObject;
use async_graphql::Object;

#[derive(Default)]
pub struct TransperthQuery;

#[Object]
impl TransperthQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Transperth, Action::Read)))]
    async fn transperth(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<TransperthObject> {
        Ok(TransperthObject)
    }
}
