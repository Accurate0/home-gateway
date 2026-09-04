use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::energy_object::EnergyObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EnergyQuery;

#[Object]
impl EnergyQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Energy, Action::Read)))]
    async fn energy(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnergyObject> {
        Ok(EnergyObject {})
    }
}
