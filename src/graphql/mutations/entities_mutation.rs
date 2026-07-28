use async_graphql::Object;

use crate::device_registry::DeviceRegistry;
use crate::graphql::mutations::light_mutation::LightMutation;
use crate::graphql::mutations::roborock_mutation::RoborockMutation;
use crate::graphql::mutations::valetudo_mutation::ValetudoMutation;

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
            return Err(async_graphql::Error::new(format!(
                "unknown roborock `{id}`"
            )));
        };

        Ok(RoborockMutation {
            control_entity: settings.control_entity.clone(),
            start_service: settings.start_service.clone(),
            stop_service: settings.stop_service.clone(),
            dock_service: settings.dock_service.clone(),
        })
    }

    async fn valetudo(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<ValetudoMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        let Some(settings) = registry.valetudo(&address) else {
            return Err(async_graphql::Error::new(format!(
                "unknown valetudo `{id}`"
            )));
        };

        Ok(ValetudoMutation {
            command_topic: settings.command_topic.clone(),
            start_payload: settings.start_payload.clone(),
            stop_payload: settings.stop_payload.clone(),
            dock_payload: settings.dock_payload.clone(),
        })
    }
}
