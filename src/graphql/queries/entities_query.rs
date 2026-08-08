use async_graphql::Object;

use crate::auth::context::AuthContext;
use crate::auth::scope::required;
use crate::device_registry::DeviceRegistry;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::entity_object::{
    DoorEntity, EinkDisplayEntity, Entity, EntitySection, EnvironmentEntity, LightEntity,
    MediaPlayerEntity, PresenceEntity, RobotVacuumEntity,
};

#[derive(Default)]
pub struct EntitiesQuery;

#[Object]
impl EntitiesQuery {
    /// Every configured entity and its current state. Entity types the caller
    /// lacks a `graphql:<type>:read` scope for are silently omitted — this query
    /// never errors on missing permissions.
    /// The dashboard sections, in display order, with their titles. Categorising
    /// a device kind is a backend concern — the frontend renders whatever order
    /// and titles this returns.
    async fn entity_sections(&self) -> Vec<EntitySection> {
        EntitySection::ordered()
    }

    async fn entities(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Entity>> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let auth = ctx.data::<AuthContext>()?;

        let mut out = Vec::new();

        if auth.has(&required::GRAPHQL_LIGHT_READ) {
            out.extend(
                registry
                    .lights()
                    .filter_map(|(address, _)| LightEntity::from_registry(registry, address))
                    .map(Entity::Light),
            );
        }

        if auth.has(&required::GRAPHQL_DOOR_READ) {
            out.extend(
                registry
                    .doors()
                    .filter_map(|(address, _)| DoorEntity::from_registry(registry, address))
                    .map(Entity::Door),
            );
        }

        if auth.has(&required::GRAPHQL_PRESENCE_READ) {
            out.extend(
                registry
                    .presence_devices()
                    .filter_map(|(address, _)| PresenceEntity::from_registry(registry, address))
                    .map(Entity::Presence),
            );
        }

        if auth.has(&required::GRAPHQL_ENVIRONMENT_READ) {
            out.extend(
                registry
                    .environment_devices()
                    .filter_map(|(address, _)| EnvironmentEntity::from_registry(registry, address))
                    .map(Entity::Environment),
            );
        }

        if auth.has(&required::GRAPHQL_EPD_READ) {
            out.extend(
                registry
                    .eink_displays()
                    .keys()
                    .filter_map(|address| EinkDisplayEntity::from_firmware(registry, address))
                    .map(Entity::EinkDisplay),
            );
            out.extend(
                registry
                    .trmnl_devices()
                    .keys()
                    .filter_map(|address| EinkDisplayEntity::from_trmnl(registry, address))
                    .map(Entity::EinkDisplay),
            );
        }

        if auth.has(&required::GRAPHQL_ROBOT_VACUUM_READ) {
            out.extend(
                registry
                    .roborocks()
                    .filter_map(|(address, _)| RobotVacuumEntity::from_roborock(registry, address))
                    .map(Entity::RobotVacuum),
            );
            out.extend(
                registry
                    .valetudos()
                    .filter_map(|(address, _)| RobotVacuumEntity::from_valetudo(registry, address))
                    .map(Entity::RobotVacuum),
            );
        }

        if auth.has(&required::GRAPHQL_MEDIA_PLAYER_READ) {
            out.extend(
                registry
                    .media_players()
                    .filter_map(|(address, _)| MediaPlayerEntity::from_registry(registry, address))
                    .map(Entity::MediaPlayer),
            );
        }

        Ok(out)
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_MEDIA_PLAYER_READ))]
    async fn media_player(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<MediaPlayerEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        MediaPlayerEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown media player `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_READ))]
    async fn light(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<LightEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        LightEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown light `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_DOOR_READ))]
    async fn door(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<DoorEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        DoorEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown door `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_PRESENCE_READ))]
    async fn presence(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<PresenceEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        PresenceEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown presence sensor `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ENVIRONMENT_READ))]
    async fn environment(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<EnvironmentEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        EnvironmentEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown environment sensor `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_EPD_READ))]
    async fn eink_display(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<EinkDisplayEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        EinkDisplayEntity::from_firmware(registry, &address)
            .or_else(|| EinkDisplayEntity::from_trmnl(registry, &address))
            .ok_or_else(|| async_graphql::Error::new(format!("unknown eink display `{id}`")))
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOT_VACUUM_READ))]
    async fn robot_vacuum(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<RobotVacuumEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.address_or_self(&id).to_owned();
        RobotVacuumEntity::from_roborock(registry, &address)
            .or_else(|| RobotVacuumEntity::from_valetudo(registry, &address))
            .ok_or_else(|| async_graphql::Error::new(format!("unknown robot vacuum `{id}`")))
    }
}
