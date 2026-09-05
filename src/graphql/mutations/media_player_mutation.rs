use async_graphql::Object;
use serde_json::json;

use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::integrations::home_assistant::HomeAssistant;
use crate::settings::MediaPlayerSettings;

pub struct MediaPlayerMutation {
    entity_id: String,
}

impl MediaPlayerMutation {
    pub fn new(settings: &MediaPlayerSettings) -> Self {
        Self {
            entity_id: settings.entity_id.clone(),
        }
    }

    async fn call(
        &self,
        ctx: &async_graphql::Context<'_>,
        service: &str,
        extra: serde_json::Value,
    ) -> async_graphql::Result<bool> {
        let home_assistant = crate::graphql::require::<HomeAssistant>(ctx, "home assistant")?;

        let mut data = json!({ "entity_id": self.entity_id });
        if let (Some(target), Some(extra)) = (data.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                target.insert(key.clone(), value.clone());
            }
        }

        home_assistant
            .call_service("media_player", service, data)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }
}

#[Object]
impl MediaPlayerMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn play(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_play", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn pause(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_pause", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn play_pause(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_play_pause", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_stop", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn next(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_next_track", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn previous(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "media_previous_track", json!({})).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn volume_set(
        &self,
        ctx: &async_graphql::Context<'_>,
        level: f64,
    ) -> async_graphql::Result<bool> {
        if !(0.0..=1.0).contains(&level) {
            return Err(async_graphql::Error::new(format!(
                "volume level {level} is outside the 0.0..=1.0 range home assistant accepts"
            )));
        }

        self.call(ctx, "volume_set", json!({ "volume_level": level }))
            .await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn mute(
        &self,
        ctx: &async_graphql::Context<'_>,
        muted: bool,
    ) -> async_graphql::Result<bool> {
        self.call(ctx, "volume_mute", json!({ "is_volume_muted": muted }))
            .await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::MediaPlayer, Action::Write)))]
    async fn turn_off(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.call(ctx, "turn_off", json!({})).await
    }
}
