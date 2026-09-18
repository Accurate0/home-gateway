use mlua::{Lua, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const PLAYER_STATE: LuaClass = LuaClass {
    name: "MediaPlayerState",
    fields: &[
        LuaField {
            name: "state",
            ty: LuaType::String,
        },
        LuaField {
            name: "app",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "source",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "title",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "series",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "content_type",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "volume",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "muted",
            ty: LuaType::Optional(&LuaType::Boolean),
        },
        LuaField {
            name: "updated_at",
            ty: LuaType::Integer,
        },
    ],
};

const PLAYER: LuaFunction = LuaFunction {
    name: "player",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Optional(&LuaType::Class(&PLAYER_STATE))),
    scope: Some(Scope::new(Resource::MediaPlayer, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[PLAYER];

pub struct MediaLua;

impl LuaModule for MediaLua {
    fn namespace(&self) -> &'static str {
        "media"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let player_cx = cx.clone();
        cx.expose(table, &PLAYER, || {
            lua.create_async_function(move |lua, device: String| {
                let cx = player_cx.clone();

                async move {
                    let address = cx.state.devices.address_or_self(&device).to_owned();
                    let mut keys = vec![device];

                    if !keys.contains(&address) {
                        keys.push(address);
                    }

                    let rows = cx
                        .query("media.player", || async {
                            cx.state.repos.media_player().latest_many(&keys).await
                        })
                        .await?;

                    let Some(row) = rows.into_iter().max_by_key(|row| row.updated_at) else {
                        return Ok(LuaValue::Nil);
                    };

                    let result = lua.create_table()?;

                    result.set("state", row.state)?;
                    result.set("app", row.app_name)?;
                    result.set("source", row.source)?;
                    result.set("title", row.media_title)?;
                    result.set("series", row.media_series_title)?;
                    result.set("content_type", row.media_content_type)?;
                    result.set("volume", row.volume_level)?;
                    result.set("muted", row.muted)?;
                    result.set("updated_at", row.updated_at.timestamp())?;

                    Ok(LuaValue::Table(result))
                }
            })
        })
    }
}
