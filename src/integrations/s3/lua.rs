use std::io::Error;

use mlua::{Lua, LuaString};

use crate::lua::{LuaCallContext, lua_module};

use super::S3;

const PREFIX: &str = "lua/";

pub struct S3Lua;

fn scoped(key: &str) -> String {
    format!("{PREFIX}{}", key.trim_start_matches('/'))
}

#[lua_module(namespace = "s3", requires = S3)]
impl S3Lua {
    #[lua(scope = S3::Read)]
    async fn get(cx: &LuaCallContext, lua: &Lua, key: String) -> mlua::Result<Option<LuaString>> {
        let s3 = cx.state.handles.expect::<S3>();
        let key = scoped(&key);

        let exists = cx
            .query(Self::GET, || async {
                s3.get_object_metadata(&key).await.map_err(Error::other)
            })
            .await?;

        if exists.is_none() {
            return Ok(None);
        }

        let payload = cx
            .query(Self::GET, || async {
                s3.get_object(&key).await.map_err(Error::other)
            })
            .await?;

        Ok(Some(lua.create_string(payload)?))
    }

    #[lua(scope = S3::Write)]
    async fn put(
        cx: &LuaCallContext,
        key: String,
        body: LuaString,
        content_type: Option<String>,
    ) -> mlua::Result<()> {
        let s3 = cx.state.handles.expect::<S3>();
        let key = scoped(&key);
        let payload = body.as_bytes().to_vec();

        cx.command(
            Self::PUT,
            format!("{key} ({} bytes)", payload.len()),
            || async {
                s3.put_object(&key, &payload, content_type.as_deref())
                    .await
                    .map_err(Error::other)
            },
        )
        .await
    }

    #[lua(scope = S3::Read)]
    async fn list(cx: &LuaCallContext, prefix: String) -> mlua::Result<Vec<String>> {
        let s3 = cx.state.handles.expect::<S3>();
        let prefix = scoped(&prefix);

        let keys = cx
            .query(Self::LIST, || async {
                s3.list_objects(&prefix).await.map_err(Error::other)
            })
            .await?;

        Ok(keys
            .into_iter()
            .map(|key| key.strip_prefix(PREFIX).unwrap_or(&key).to_owned())
            .collect())
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
