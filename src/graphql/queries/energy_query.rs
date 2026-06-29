use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::energy_object::EnergyObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EnergyQuery;

#[Object]
impl EnergyQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_ENERGY_READ))]
    async fn energy(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnergyObject> {
        Ok(EnergyObject {})
    }
}
