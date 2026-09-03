use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::fuelwatch_object::FuelWatchObject;
use async_graphql::Object;

#[derive(Default)]
pub struct FuelWatchQuery;

#[Object]
impl FuelWatchQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_FUELWATCH_READ))]
    async fn fuelwatch(
        &self,
        _ctx: &async_graphql::Context<'_>,
        postcode: Option<i32>,
    ) -> async_graphql::Result<FuelWatchObject> {
        Ok(FuelWatchObject { postcode })
    }
}
