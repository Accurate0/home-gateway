use async_graphql::Object;

use crate::device_registry::DeviceRegistry;
use crate::graphql::mutations::light_mutation::LightMutation;
use crate::graphql::mutations::robot_vacuum_mutation::RobotVacuumMutation;

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

    async fn robot_vacuum(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<RobotVacuumMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();

        if let Some(settings) = registry.roborock(&address) {
            return Ok(RobotVacuumMutation::roborock(settings));
        }

        if let Some(settings) = registry.valetudo(&address) {
            return Ok(RobotVacuumMutation::valetudo(settings));
        }

        Err(async_graphql::Error::new(format!(
            "unknown robot vacuum `{id}`"
        )))
    }
}
