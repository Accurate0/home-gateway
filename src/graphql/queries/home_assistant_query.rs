use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::home_assistant_object::HomeAssistantObject;
use async_graphql::Object;

#[derive(Default)]
pub struct HomeAssistantQuery;

#[Object]
impl HomeAssistantQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::HomeAssistant, Action::Read)))]
    async fn home_assistant(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<HomeAssistantObject> {
        Ok(HomeAssistantObject {})
    }
}
