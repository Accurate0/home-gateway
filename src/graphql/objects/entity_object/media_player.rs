use async_graphql::{Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::device_registry::DeviceRegistry;
use crate::graphql::dataloader::media_player_state::{
    MediaPlayerStateDataLoader, MediaPlayerStateModel,
};

pub struct MediaPlayerEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
}

impl MediaPlayerEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.media_player(address)?;

        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
        })
    }

    async fn model(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<MediaPlayerStateModel>> {
        let loader = ctx.data::<DataLoader<MediaPlayerStateDataLoader>>()?;

        Ok(loader.load_one(self.id.clone()).await?)
    }
}

#[Object]
impl MediaPlayerEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Media
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.map(|m| m.state))
    }

    async fn playing(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        Ok(self
            .model(ctx)
            .await?
            .is_some_and(|m| m.state == "playing" || m.state == "buffering"))
    }

    async fn app_name(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.app_name))
    }

    async fn source(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.source))
    }

    async fn media_content_type(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_content_type))
    }

    async fn media_title(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_title))
    }

    async fn media_artist(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_artist))
    }

    async fn media_album_name(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_album_name))
    }

    async fn media_series_title(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_series_title))
    }

    async fn season(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<i32>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_season))
    }

    async fn episode(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<i32>> {
        Ok(self.model(ctx).await?.and_then(|m| m.media_episode))
    }

    async fn duration_seconds(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self.model(ctx).await?.and_then(|m| m.duration_seconds))
    }

    /// Where playback has reached now, extrapolated from the last reported
    /// position: Home Assistant only republishes `media_position` on a seek, so a
    /// stored value goes stale the moment it lands.
    async fn position_seconds(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let Some(model) = self.model(ctx).await? else {
            return Ok(None);
        };

        let Some(position) = model.position_seconds else {
            return Ok(None);
        };

        let elapsed = match (model.state.as_str(), model.position_updated_at) {
            ("playing" | "buffering", Some(updated_at)) => {
                (Utc::now() - updated_at).num_milliseconds() as f64 / 1000.0
            }
            _ => 0.0,
        };

        let position = position + elapsed.max(0.0);

        Ok(Some(match model.duration_seconds {
            Some(duration) => position.min(duration),
            None => position,
        }))
    }

    async fn progress(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let Some(model) = self.model(ctx).await? else {
            return Ok(None);
        };

        let position = self.position_seconds(ctx).await?;

        Ok(match (position, model.duration_seconds) {
            (Some(position), Some(duration)) if duration > 0.0 => Some(position / duration),
            _ => None,
        })
    }

    async fn volume_level(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(self.model(ctx).await?.and_then(|m| m.volume_level))
    }

    async fn muted(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<bool>> {
        Ok(self.model(ctx).await?.and_then(|m| m.muted))
    }

    async fn artwork_url(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(self.model(ctx).await?.and_then(|m| m.artwork_url))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(self.model(ctx).await?.map(|m| m.updated_at))
    }
}
