use chrono::Utc;
use mlua::{Lua, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

use super::Transperth;

const DEPARTURE: LuaClass = LuaClass {
    name: "TransperthDeparture",
    fields: &[
        LuaField {
            name: "line",
            ty: LuaType::String,
        },
        LuaField {
            name: "headsign",
            ty: LuaType::String,
        },
        LuaField {
            name: "platform",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "minutes",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "delay",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaField {
            name: "live",
            ty: LuaType::Boolean,
        },
    ],
};

const NEXT: LuaFunction = LuaFunction {
    name: "next",
    params: &[LuaParam {
        name: "route",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Optional(&LuaType::Array(&LuaType::Class(
        &DEPARTURE,
    )))),
    scope: Some(Scope::new(Resource::Transperth, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[NEXT];

pub struct TransperthLua;

impl LuaModule for TransperthLua {
    fn namespace(&self) -> &'static str {
        "transperth"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        if !cx.state.handles.contains::<Transperth>() {
            return Ok(());
        }

        let next_cx = cx.clone();
        cx.expose(table, &NEXT, || {
            lua.create_async_function(move |lua, route: String| {
                let cx = next_cx.clone();

                async move {
                    let transperth = cx.state.handles.expect::<Transperth>().clone();

                    let Some(departures) = transperth.route(&route).await else {
                        tracing::info!(
                            "[{}] lua transperth.next has no departures cached for {route}",
                            cx.event_id
                        );

                        return Ok(LuaValue::Nil);
                    };

                    let now = Utc::now();
                    let result = lua.create_table()?;

                    for departure in departures
                        .departures
                        .iter()
                        .filter(|departure| departure.minutes_away(now) >= 0)
                    {
                        let row = lua.create_table()?;

                        row.set("line", departure.line.as_str())?;
                        row.set("headsign", departure.headsign.as_str())?;
                        row.set("platform", departure.platform.as_deref())?;
                        row.set("minutes", departure.minutes_away(now))?;
                        row.set("delay", departure.delay_minutes())?;
                        row.set("live", departure.live_departure.is_some())?;

                        result.push(row)?;
                    }

                    Ok(LuaValue::Table(result))
                }
            })
        })
    }
}
