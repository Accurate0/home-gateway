use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::transperth_object::TransperthObject;
use async_graphql::Object;

#[derive(Default)]
pub struct TransperthQuery;

#[Object]
impl TransperthQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_TRANSPERTH_READ))]
    async fn transperth(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<TransperthObject> {
        Ok(TransperthObject)
    }
}
