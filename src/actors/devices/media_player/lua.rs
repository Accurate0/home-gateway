use crate::lua::{LuaCallContext, LuaClass, lua_module};
use crate::repo::jellyfin::JellyfinSessionRow;
use crate::repo::media_player::MediaPlayerStateRow;

#[derive(LuaClass)]
#[lua(output)]
pub struct JellyfinSession {
    user: String,
    device: String,
    client: String,
    item: String,
    item_type: String,
    series: Option<String>,
    season: Option<i32>,
    episode: Option<i32>,
    position: Option<f64>,
    runtime: Option<f64>,
    paused: bool,
}

impl From<JellyfinSessionRow> for JellyfinSession {
    fn from(row: JellyfinSessionRow) -> Self {
        JellyfinSession {
            user: row.user_name,
            device: row.device_name,
            client: row.client,
            item: row.item_name,
            item_type: row.item_type,
            series: row.series_name,
            season: row.season,
            episode: row.episode,
            position: row.position_seconds,
            runtime: row.runtime_seconds,
            paused: row.paused,
        }
    }
}

#[derive(LuaClass)]
#[lua(output)]
pub struct MediaPlayerState {
    state: String,
    app: Option<String>,
    source: Option<String>,
    title: Option<String>,
    series: Option<String>,
    content_type: Option<String>,
    volume: Option<f64>,
    muted: Option<bool>,
    updated_at: i64,
}

impl From<MediaPlayerStateRow> for MediaPlayerState {
    fn from(row: MediaPlayerStateRow) -> Self {
        MediaPlayerState {
            state: row.state,
            app: row.app_name,
            source: row.source,
            title: row.media_title,
            series: row.media_series_title,
            content_type: row.media_content_type,
            volume: row.volume_level,
            muted: row.muted,
            updated_at: row.updated_at.timestamp(),
        }
    }
}

pub struct MediaLua;

#[lua_module(namespace = "media")]
impl MediaLua {
    #[lua(scope = Jellyfin::Read)]
    async fn jellyfin(cx: &LuaCallContext) -> mlua::Result<Vec<JellyfinSession>> {
        let sessions = cx
            .query(Self::JELLYFIN, || async {
                cx.state.repos.jellyfin().sessions().await
            })
            .await?;

        Ok(sessions.into_iter().map(JellyfinSession::from).collect())
    }

    #[lua(scope = MediaPlayer::Read)]
    async fn player(cx: &LuaCallContext, device: String) -> mlua::Result<Option<MediaPlayerState>> {
        let address = cx.state.devices.address_or_self(&device).to_owned();
        let mut keys = vec![device];

        if !keys.contains(&address) {
            keys.push(address);
        }

        let rows = cx
            .query(Self::PLAYER, || async {
                cx.state.repos.media_player().latest_many(&keys).await
            })
            .await?;

        let latest = rows.into_iter().max_by_key(|row| row.updated_at);

        Ok(latest.map(MediaPlayerState::from))
    }
}
