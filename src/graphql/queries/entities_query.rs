use async_graphql::Object;

use crate::auth::context::AuthContext;
use crate::auth::scope::{Action, Resource, Scope};
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

        if auth.has(&Scope::new(Resource::Light, Action::Read)) {
            out.extend(
                registry
                    .lights()
                    .filter_map(|(address, _)| LightEntity::from_registry(registry, address))
                    .map(Entity::Light),
            );
        }

        if auth.has(&Scope::new(Resource::Door, Action::Read)) {
            out.extend(
                registry
                    .doors()
                    .filter_map(|(address, _)| DoorEntity::from_registry(registry, address))
                    .map(Entity::Door),
            );
        }

        if auth.has(&Scope::new(Resource::Presence, Action::Read)) {
            out.extend(
                registry
                    .presence_devices()
                    .filter_map(|(address, _)| PresenceEntity::from_registry(registry, address))
                    .map(Entity::Presence),
            );
        }

        if auth.has(&Scope::new(Resource::Environment, Action::Read)) {
            out.extend(
                registry
                    .environment_devices()
                    .filter_map(|(address, _)| EnvironmentEntity::from_registry(registry, address))
                    .map(Entity::Environment),
            );
        }

        if auth.has(&Scope::new(Resource::Epd, Action::Read)) {
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

        if auth.has(&Scope::new(Resource::RobotVacuum, Action::Read)) {
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

        if auth.has(&Scope::new(Resource::MediaPlayer, Action::Read)) {
            out.extend(
                registry
                    .media_players()
                    .filter_map(|(address, _)| MediaPlayerEntity::from_registry(registry, address))
                    .map(Entity::MediaPlayer),
            );
        }

        Ok(out)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Door, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Presence, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Environment, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Epd, Action::Read)))]
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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::RobotVacuum, Action::Read)))]
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
