use async_graphql::Object;

use crate::auth::scope::{Action, Resource, Scope};
use crate::device_command::CommandTargets;
use crate::device_registry::DeviceRegistry;
use crate::graphql::guard::ScopeGuard;
use crate::media_control::{self, MediaCommand};
use crate::settings::MediaPlayerSettings;
use crate::state::HandleRegistry;

pub struct MediaPlayerMutation {
    address: String,
}

impl MediaPlayerMutation {
    pub fn new(settings: &MediaPlayerSettings) -> Self {
        Self {
            address: settings.address.clone(),
        }
    }

    async fn run(
        &self,
        ctx: &async_graphql::Context<'_>,
        command: MediaCommand,
    ) -> async_graphql::Result<bool> {
        let targets =
            CommandTargets::new(ctx.data::<DeviceRegistry>()?, ctx.data::<HandleRegistry>()?);

        media_control::send(&targets, &self.address, command)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }
}

#[Object]
impl MediaPlayerMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn play(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Play).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn pause(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Pause).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn play_pause(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::PlayPause).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Stop).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn next(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Next).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn previous(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Previous).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn play_media(
        &self,
        ctx: &async_graphql::Context<'_>,
        url: String,
        #[graphql(default)] announcement: bool,
    ) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::PlayMedia { url, announcement })
            .await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn volume_set(
        &self,
        ctx: &async_graphql::Context<'_>,
        level: f64,
    ) -> async_graphql::Result<bool> {
        if !(0.0..=1.0).contains(&level) {
            return Err(async_graphql::Error::new(format!(
                "volume level {level} is outside the 0.0..=1.0 range media players accept"
            )));
        }

        self.run(ctx, MediaCommand::Volume(level)).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn mute(
        &self,
        ctx: &async_graphql::Context<'_>,
        muted: bool,
    ) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::Mute(muted)).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn turn_off(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, MediaCommand::TurnOff).await
    }
}
