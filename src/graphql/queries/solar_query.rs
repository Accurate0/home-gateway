use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::solar_object::SolarObject;
use async_graphql::Object;

#[derive(Default)]
pub struct SolarQuery;

#[Object(name = "HomeSolarQuery")]
impl SolarQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Solar, Action::Read)))]
    async fn home_solar(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<SolarObject> {
        Ok(SolarObject {})
    }
}
