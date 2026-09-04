use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::fuelwatch_object::FuelWatchObject;
use async_graphql::Object;

#[derive(Default)]
pub struct FuelWatchQuery;

#[Object]
impl FuelWatchQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::FuelWatch, Action::Read)))]
    async fn fuelwatch(
        &self,
        _ctx: &async_graphql::Context<'_>,
        postcode: Option<i32>,
    ) -> async_graphql::Result<FuelWatchObject> {
        Ok(FuelWatchObject { postcode })
    }
}
