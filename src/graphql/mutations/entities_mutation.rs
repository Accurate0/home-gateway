use async_graphql::Object;

use crate::device_registry::{DeviceRegistry, IdOrAlias};
use crate::graphql::mutations::air_purifier_mutation::AirPurifierMutation;
use crate::graphql::mutations::eink_display_mutation::EinkDisplayMutation;
use crate::graphql::mutations::garage_door_mutation::GarageDoorMutation;
use crate::graphql::mutations::light_mutation::LightMutation;
use crate::graphql::mutations::media_player_mutation::MediaPlayerMutation;
use crate::graphql::mutations::robot_vacuum_mutation::RobotVacuumMutation;

#[derive(Default)]
pub struct EntitiesMutation;

#[Object]
impl EntitiesMutation {
    async fn light(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<LightMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        let capabilities = registry.capabilities(&address).to_vec();
        Ok(LightMutation {
            address,
            capabilities,
        })
    }

    async fn robot_vacuum(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<RobotVacuumMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();

        if let Some(settings) = registry.robot_vacuum(&address) {
            return Ok(RobotVacuumMutation::new(settings));
        }

        Err(async_graphql::Error::new(format!(
            "unknown robot vacuum `{id}`"
        )))
    }

    async fn garage_door(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<GarageDoorMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();

        if registry.garage_door(&address).is_none() {
            return Err(async_graphql::Error::new(format!(
                "unknown garage door `{id}`"
            )));
        }

        Ok(GarageDoorMutation::new(address))
    }

    async fn air_purifier(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<AirPurifierMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();

        if registry.air_purifier(&address).is_none() {
            return Err(async_graphql::Error::new(format!(
                "unknown air purifier `{id}`"
            )));
        }

        Ok(AirPurifierMutation::new(address))
    }

    async fn media_player(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<MediaPlayerMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();

        let Some(settings) = registry.media_player(&address) else {
            return Err(async_graphql::Error::new(format!(
                "unknown media player `{id}`"
            )));
        };

        Ok(MediaPlayerMutation::new(settings))
    }

    async fn eink_display(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<EinkDisplayMutation> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();

        if registry.eink_display(&address).is_none() {
            return Err(async_graphql::Error::new(format!(
                "unknown eink display `{id}`"
            )));
        }

        Ok(EinkDisplayMutation { address })
    }
}
