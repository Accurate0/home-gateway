use crate::actors::devices::handler::DeviceHandler;
use std::collections::HashMap;

use uuid::Uuid;

use crate::event_bus::EventBusMessage;
use crate::state::SharedActorState;

pub mod state;

use state::{Attributes, Prior, edges};

pub struct Update {
    pub event_id: Uuid,
    pub address: String,
    pub state: String,
    pub attributes: serde_json::Value,
}

pub enum Message {
    HomeAssistant(Update),
}

pub struct MediaPlayerHandler {
    shared_actor_state: SharedActorState,
}

impl MediaPlayerHandler {
    pub const NAME: &str = "media_player";

    async fn handle_update(
        &self,
        update: Update,
        priors: &mut HashMap<String, Prior>,
    ) -> Result<(), anyhow::Error> {
        let Update {
            event_id,
            address,
            state,
            attributes,
        } = update;

        let devices = &self.shared_actor_state.devices;
        let Some(settings) = devices.media_player(&address) else {
            tracing::warn!("media player update for unregistered entity {address}");
            return Ok(());
        };

        let device_id = settings.id.clone();
        let name = settings.name.clone();
        let room = devices.room(&address).map(str::to_owned);

        let parsed = match serde_json::from_value::<Attributes>(attributes.clone()) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!("failed to parse media player attributes for {address}: {e}");
                Attributes::default()
            }
        };
        let raw_attributes = attributes;
        let attributes = parsed;

        let artwork_url = attributes.entity_picture.as_deref().and_then(|path| {
            self.shared_actor_state
                .home_assistant
                .as_ref()
                .map(|ha| ha.absolute_url(path))
        });

        let now = Prior {
            state: state.clone(),
            media_title: attributes.media_title.clone(),
        };

        let prior = match priors.get(&device_id) {
            Some(prior) => Some(prior.clone()),
            None => self.load_prior(&device_id).await?,
        };

        let edges = edges(prior.as_ref(), &now);

        tracing::debug!(
            "media player {device_id} is {state} ({}), app {}, position {}/{}",
            attributes.media_title.as_deref().unwrap_or("nothing"),
            attributes.app_name.as_deref().unwrap_or("unknown"),
            attributes.media_position.unwrap_or_default(),
            attributes.media_duration.unwrap_or_default(),
        );

        priors.insert(device_id.clone(), now);

        self.save_state(
            event_id,
            &device_id,
            &state,
            &attributes,
            artwork_url.as_deref(),
            &raw_attributes,
        )
        .await?;

        for edge in edges {
            tracing::info!(
                "media player {device_id} {} {}",
                edge.as_str(),
                attributes.media_title.as_deref().unwrap_or("playback"),
            );

            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::MediaPlayer {
                    event_id,
                    device_id: device_id.clone(),
                    name: name.clone(),
                    room: room.clone(),
                    state: edge,
                    entity_state: state.clone(),
                    app_name: attributes.app_name.clone(),
                    source: attributes.source.clone(),
                    media_title: attributes.media_title.clone(),
                    media_series_title: attributes.media_series_title.clone(),
                    media_content_type: attributes.media_content_type.clone(),
                    season: attributes.media_season,
                    episode: attributes.media_episode,
                    position_seconds: attributes.media_position,
                    duration_seconds: attributes.media_duration,
                    volume_level: attributes.volume_level,
                    muted: attributes.is_volume_muted,
                    artwork_url: artwork_url.clone(),
                });
        }

        Ok(())
    }

    async fn load_prior(&self, device_id: &str) -> Result<Option<Prior>, anyhow::Error> {
        let row = sqlx::query!(
            "SELECT state, media_title FROM media_player_state WHERE device_id = $1",
            device_id,
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(row.map(|row| Prior {
            state: row.state,
            media_title: row.media_title,
        }))
    }

    async fn save_state(
        &self,
        event_id: Uuid,
        device_id: &str,
        state: &str,
        attributes: &Attributes,
        artwork_url: Option<&str>,
        raw_attributes: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"INSERT INTO media_player_state (
                device_id, state, app_name, source, media_content_type, media_title,
                media_artist, media_album_name, media_series_title, media_season, media_episode,
                position_seconds, duration_seconds, position_updated_at, volume_level, muted,
                artwork_url, attributes, event_id, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, now())
            ON CONFLICT (device_id) DO UPDATE SET
                state = EXCLUDED.state,
                app_name = EXCLUDED.app_name,
                source = EXCLUDED.source,
                media_content_type = EXCLUDED.media_content_type,
                media_title = EXCLUDED.media_title,
                media_artist = EXCLUDED.media_artist,
                media_album_name = EXCLUDED.media_album_name,
                media_series_title = EXCLUDED.media_series_title,
                media_season = EXCLUDED.media_season,
                media_episode = EXCLUDED.media_episode,
                position_seconds = EXCLUDED.position_seconds,
                duration_seconds = EXCLUDED.duration_seconds,
                position_updated_at = EXCLUDED.position_updated_at,
                volume_level = COALESCE(EXCLUDED.volume_level, media_player_state.volume_level),
                muted = COALESCE(EXCLUDED.muted, media_player_state.muted),
                artwork_url = EXCLUDED.artwork_url,
                attributes = EXCLUDED.attributes,
                event_id = EXCLUDED.event_id,
                updated_at = now()
            "#,
            device_id,
            state,
            attributes.app_name,
            attributes.source,
            attributes.media_content_type,
            attributes.media_title,
            attributes.media_artist,
            attributes.media_album_name,
            attributes.media_series_title,
            attributes.media_season,
            attributes.media_episode,
            attributes.media_position,
            attributes.media_duration,
            attributes.media_position_updated_at,
            attributes.volume_level,
            attributes.is_volume_muted,
            artwork_url,
            raw_attributes,
            event_id,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }
}

impl DeviceHandler for MediaPlayerHandler {
    const NAME: &'static str = MediaPlayerHandler::NAME;

    type Message = Message;
    type State = HashMap<String, Prior>;

    fn new(shared_actor_state: SharedActorState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        match message {
            Message::HomeAssistant(update) => self.handle_update(update, state).await,
        }
    }
}
