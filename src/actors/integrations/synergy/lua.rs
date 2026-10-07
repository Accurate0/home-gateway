use chrono::DateTime;
use mlua::ExternalError;

use crate::lua::{LuaCallContext, LuaClass, lua_module};

#[derive(LuaClass)]
#[lua(output)]
pub struct EnergyInterval {
    used: f64,
    exported: f64,
    at: i64,
}

pub struct EnergyLua;

#[lua_module(namespace = "energy")]
impl EnergyLua {
    #[lua(scope = Energy::Read)]
    async fn since(cx: &LuaCallContext, epoch: i64) -> mlua::Result<Vec<EnergyInterval>> {
        let since = DateTime::from_timestamp(epoch, 0)
            .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

        let rows = cx
            .query(Self::SINCE, || async {
                cx.state.repos.energy().history_since(since).await
            })
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| EnergyInterval {
                used: row.energy_used,
                exported: row.solar_exported,
                at: row.time.timestamp(),
            })
            .collect())
    }
}
