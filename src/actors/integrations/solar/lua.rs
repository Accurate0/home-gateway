use crate::integrations::solar;
use crate::lua::{LuaCallContext, LuaClass, lua_module};

#[derive(LuaClass)]
#[lua(output)]
pub struct SolarAverages {
    last_15_mins: Option<f64>,
    last_1_hour: Option<f64>,
    last_3_hours: Option<f64>,
}

pub struct SolarLua;

#[lua_module(namespace = "solar")]
impl SolarLua {
    #[lua(scope = Solar::Read)]
    async fn current(cx: &LuaCallContext) -> mlua::Result<Option<f64>> {
        cx.query(Self::CURRENT, || async {
            solar::queries::current_wh(&cx.state.db).await
        })
        .await
    }

    #[lua(scope = Solar::Read)]
    async fn averages(cx: &LuaCallContext) -> mlua::Result<SolarAverages> {
        let statistics = cx
            .query(Self::AVERAGES, || async {
                solar::queries::statistics(&cx.state.db).await
            })
            .await?;

        Ok(SolarAverages {
            last_15_mins: statistics.averages.last_15_mins,
            last_1_hour: statistics.averages.last_1_hour,
            last_3_hours: statistics.averages.last_3_hours,
        })
    }
}
