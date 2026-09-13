use mlua::{Lua, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

use super::S3;

const PREFIX: &str = "lua/";

const KEY: LuaParam = LuaParam {
    name: "key",
    ty: LuaType::String,
};

const GET: LuaFunction = LuaFunction {
    name: "get",
    params: &[KEY],
    returns: Some(LuaType::Optional(&LuaType::String)),
    scope: Some(Scope::new(Resource::S3, Action::Read)),
};

const PUT: LuaFunction = LuaFunction {
    name: "put",
    params: &[
        KEY,
        LuaParam {
            name: "body",
            ty: LuaType::String,
        },
        LuaParam {
            name: "content_type",
            ty: LuaType::Optional(&LuaType::String),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::S3, Action::Write)),
};

const LIST: LuaFunction = LuaFunction {
    name: "list",
    params: &[LuaParam {
        name: "prefix",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Array(&LuaType::String)),
    scope: Some(Scope::new(Resource::S3, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[GET, PUT, LIST];

pub struct S3Lua;

fn scoped(key: &str) -> String {
    format!("{PREFIX}{}", key.trim_start_matches('/'))
}

impl LuaModule for S3Lua {
    fn namespace(&self) -> &'static str {
        "s3"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        if !cx.state.handles.contains::<S3>() {
            return Ok(());
        }

        let get_cx = cx.clone();
        cx.expose(table, &GET, || {
            lua.create_async_function(move |lua, key: String| {
                let cx = get_cx.clone();

                async move {
                    let s3 = cx.state.handles.expect::<S3>();
                    let key = scoped(&key);

                    let exists = cx
                        .query("s3.get", || async {
                            s3.get_object_metadata(&key)
                                .await
                                .map_err(std::io::Error::other)
                        })
                        .await?;

                    if exists.is_none() {
                        return Ok(LuaValue::Nil);
                    }

                    let payload = cx
                        .query("s3.get", || async {
                            s3.get_object(&key).await.map_err(std::io::Error::other)
                        })
                        .await?;

                    Ok(LuaValue::String(lua.create_string(payload)?))
                }
            })
        })?;

        let put_cx = cx.clone();
        cx.expose(table, &PUT, || {
            lua.create_async_function(
                move |_, (key, body, content_type): (String, mlua::LuaString, Option<String>)| {
                    let cx = put_cx.clone();

                    async move {
                        let s3 = cx.state.handles.expect::<S3>();
                        let key = scoped(&key);
                        let payload = body.as_bytes().to_vec();

                        cx.command(
                            "s3.put",
                            format!("{key} ({} bytes)", payload.len()),
                            || async {
                                s3.put_object(&key, &payload, content_type.as_deref())
                                    .await
                                    .map_err(std::io::Error::other)
                            },
                        )
                        .await
                    }
                },
            )
        })?;

        let list_cx = cx.clone();
        cx.expose(table, &LIST, || {
            lua.create_async_function(move |_, prefix: String| {
                let cx = list_cx.clone();

                async move {
                    let s3 = cx.state.handles.expect::<S3>();
                    let prefix = scoped(&prefix);

                    let keys = cx
                        .query("s3.list", || async {
                            s3.list_objects(&prefix)
                                .await
                                .map_err(std::io::Error::other)
                        })
                        .await?;

                    Ok(keys
                        .into_iter()
                        .map(|key| key.strip_prefix(PREFIX).unwrap_or(&key).to_owned())
                        .collect::<Vec<_>>())
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::scoped;

    #[test]
    fn keys_are_confined_to_the_lua_prefix() {
        assert_eq!(scoped("notes/today.txt"), "lua/notes/today.txt");
        assert_eq!(scoped("/notes/today.txt"), "lua/notes/today.txt");
    }
}
