use crate::lua::{LuaCallContext, LuaClass, lua_module};
use crate::repo::unifi::UnifiState;

#[derive(LuaClass)]
#[lua(output)]
pub struct UnifiClient {
    name: String,
    connected: bool,
    since: i64,
}

pub struct UnifiLua;

#[lua_module(namespace = "unifi")]
impl UnifiLua {
    #[lua(scope = Unifi::Read)]
    async fn home(cx: &LuaCallContext, client: String) -> mlua::Result<bool> {
        let rows = cx
            .query(Self::HOME, || async {
                cx.state.repos.unifi().latest_states().await
            })
            .await?;

        Ok(rows
            .iter()
            .any(|row| row.name == client && row.state == UnifiState::Connected))
    }

    #[lua(scope = Unifi::Read)]
    async fn clients(cx: &LuaCallContext) -> mlua::Result<Vec<UnifiClient>> {
        let rows = cx
            .query(Self::CLIENTS, || async {
                cx.state.repos.unifi().latest_states().await
            })
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| UnifiClient {
                connected: row.state == UnifiState::Connected,
                since: row.time.timestamp(),
                name: row.name,
            })
            .collect())
    }
}
