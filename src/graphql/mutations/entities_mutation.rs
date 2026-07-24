use async_graphql::Object;

use crate::device_registry::DeviceRegistry;
use crate::graphql::mutations::light_mutation::LightMutation;
use crate::graphql::mutations::roborock_mutation::RoborockMutation;

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

    async fn roborock(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<RoborockMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        let Some(settings) = registry.roborock(&address) else {
            return Err(async_graphql::Error::new(format!("unknown roborock `{id}`")));
        };

        Ok(RoborockMutation {
            control_entity: settings.control_entity.clone(),
            stop_service: settings.stop_service.clone(),
            dock_service: settings.dock_service.clone(),
        })
    }
}
