use chrono::Utc;

use crate::lua::{LuaCallContext, LuaClass, lua_module};

use super::Transperth;

#[derive(LuaClass)]
#[lua(output)]
pub struct TransperthDeparture {
    line: String,
    headsign: String,
    platform: Option<String>,
    minutes: i64,
    delay: Option<i64>,
    live: bool,
}

pub struct TransperthLua;

#[lua_module(namespace = "transperth", requires = Transperth)]
impl TransperthLua {
    #[lua(scope = Transperth::Read)]
    async fn next(
        cx: &LuaCallContext,
        route: String,
    ) -> mlua::Result<Option<Vec<TransperthDeparture>>> {
        let transperth = cx.state.handles.expect::<Transperth>().clone();

        let Some(departures) = transperth.route(&route).await else {
            tracing::info!(
                "[{}] lua transperth.next has no departures cached for {route}",
                cx.event_id
            );

            return Ok(None);
        };

        let now = Utc::now();

        Ok(Some(
            departures
                .departures
                .iter()
                .filter(|departure| departure.minutes_away(now) >= 0)
                .map(|departure| TransperthDeparture {
                    line: departure.line.clone(),
                    headsign: departure.headsign.clone(),
                    platform: departure.platform.clone(),
                    minutes: departure.minutes_away(now),
                    delay: departure.delay_minutes(),
                    live: departure.live_departure.is_some(),
                })
                .collect(),
        ))
    }
}
