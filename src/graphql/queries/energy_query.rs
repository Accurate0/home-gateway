use crate::graphql::objects::energy_object::EnergyObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EnergyQuery;

#[Object]
impl EnergyQuery {
    async fn energy(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnergyObject> {
        Ok(EnergyObject {})
    }
}
