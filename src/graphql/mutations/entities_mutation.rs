use async_graphql::Object;

use crate::device_registry::DeviceRegistry;
use crate::graphql::mutations::light_mutation::LightMutation;

#[derive(Default)]
pub struct EntitiesMutation;

#[Object]
impl EntitiesMutation {
    async fn light(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<LightMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        let capabilities = registry.capabilities(&address).to_vec();
        Ok(LightMutation {
            address,
            capabilities,
        })
    }
}
