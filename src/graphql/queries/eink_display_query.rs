use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::eink_display_object::EinkDisplayObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EinkDisplayQuery;

#[Object]
impl EinkDisplayQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_EPD_READ))]
    async fn eink_display(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EinkDisplayObject> {
        Ok(EinkDisplayObject {})
    }
}
