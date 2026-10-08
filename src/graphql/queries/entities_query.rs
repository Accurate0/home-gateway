use async_graphql::Object;

use crate::auth::context::AuthContext;
use crate::auth::scope::{Action, Resource, Scope};
use crate::device_registry::{DeviceRegistry, IdOrAlias};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::device_connection_object::DeviceConnectionObject;
use crate::graphql::objects::entity_object::{
    AirPurifierEntity, DoorEntity, EinkDisplayEntity, Entity, EntitySection, EnvironmentEntity,
    GarageDoorEntity, LightEntity, MediaPlayerEntity, PlantEntity, PresenceEntity,
    RobotVacuumEntity,
};
use crate::repo::RepoRegistry;

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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Device, Action::Read)))]
    async fn device_connections(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<DeviceConnectionObject>> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .device()
            .connections()
            .await?
            .into_iter()
            .filter(|row| registry.address_for(&row.device_id).is_some())
            .map(|row| DeviceConnectionObject {
                device_id: row.device_id,
                connected: row.connected,
                changed_at: row.changed_at,
            })
            .collect())
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

        if auth.has(&Scope::new(Resource::Plant, Action::Read)) {
            out.extend(
                registry
                    .plant_devices()
                    .filter_map(|(address, _)| PlantEntity::from_registry(registry, address))
                    .map(Entity::Plant),
            );
        }

        if auth.has(&Scope::new(Resource::Epd, Action::Read)) {
            out.extend(
                registry
                    .eink_displays()
                    .filter_map(|(address, _)| EinkDisplayEntity::from_firmware(registry, address))
                    .map(Entity::EinkDisplay),
            );
            out.extend(
                registry
                    .trmnl_devices()
                    .filter_map(|(address, _)| EinkDisplayEntity::from_trmnl(registry, address))
                    .map(Entity::EinkDisplay),
            );
        }

        if auth.has(&Scope::new(Resource::RobotVacuum, Action::Read)) {
            out.extend(
                registry
                    .robot_vacuums()
                    .filter_map(|(address, _)| RobotVacuumEntity::from_registry(registry, address))
                    .map(Entity::RobotVacuum),
            );
        }

        if auth.has(&Scope::new(Resource::GarageDoor, Action::Read)) {
            out.extend(
                registry
                    .garage_doors()
                    .filter_map(|(address, _)| GarageDoorEntity::from_registry(registry, address))
                    .map(Entity::GarageDoor),
            );
        }

        if auth.has(&Scope::new(Resource::AirPurifier, Action::Read)) {
            out.extend(
                registry
                    .air_purifiers()
                    .filter_map(|(address, _)| AirPurifierEntity::from_registry(registry, address))
                    .map(Entity::AirPurifier),
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
        id: IdOrAlias,
    ) -> async_graphql::Result<MediaPlayerEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        MediaPlayerEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown media player `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Read)))]
    async fn light(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<LightEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        LightEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown light `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Door, Action::Read)))]
    async fn door(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<DoorEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        DoorEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown door `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Presence, Action::Read)))]
    async fn presence(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<PresenceEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        PresenceEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown presence sensor `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Environment, Action::Read)))]
    async fn environment(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<EnvironmentEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        EnvironmentEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown environment sensor `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Plant, Action::Read)))]
    async fn plant(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<PlantEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        PlantEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown plant sensor `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Epd, Action::Read)))]
    async fn eink_display(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<EinkDisplayEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        EinkDisplayEntity::from_firmware(registry, &address)
            .or_else(|| EinkDisplayEntity::from_trmnl(registry, &address))
            .ok_or_else(|| async_graphql::Error::new(format!("unknown eink display `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::RobotVacuum, Action::Read)))]
    async fn robot_vacuum(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<RobotVacuumEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        RobotVacuumEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown robot vacuum `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::GarageDoor, Action::Read)))]
    async fn garage_door(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<GarageDoorEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        GarageDoorEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown garage door `{id}`")))
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, Action::Read)))]
    async fn air_purifier(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: IdOrAlias,
    ) -> async_graphql::Result<AirPurifierEntity> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let address = registry.lookup(&id)?.address.clone();
        AirPurifierEntity::from_registry(registry, &address)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown air purifier `{id}`")))
    }
}
