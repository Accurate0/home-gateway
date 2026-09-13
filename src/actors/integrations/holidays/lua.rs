use chrono::{NaiveDate, TimeDelta, Utc, Weekday};
use chrono_tz::Australia::Perth;
use mlua::{ExternalResult, Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const DATE: LuaParam = LuaParam {
    name: "date",
    ty: LuaType::Optional(&LuaType::String),
};

const ON: LuaFunction = LuaFunction {
    name: "on",
    params: &[DATE],
    returns: Some(LuaType::Optional(&LuaType::String)),
    scope: Some(Scope::new(Resource::Holiday, Action::Read)),
};

const IN_WEEK: LuaFunction = LuaFunction {
    name: "in_week",
    params: &[DATE],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Holiday, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[ON, IN_WEEK];

pub struct HolidaysLua;

fn resolve_date(date: Option<String>) -> mlua::Result<NaiveDate> {
    match date {
        Some(date) => NaiveDate::parse_from_str(&date, "%Y-%m-%d").into_lua_err(),
        None => Ok(Utc::now().with_timezone(&Perth).date_naive()),
    }
}

impl LuaModule for HolidaysLua {
    fn namespace(&self) -> &'static str {
        "holidays"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let on_cx = cx.clone();
        cx.expose(table, &ON, || {
            lua.create_async_function(move |_, date: Option<String>| {
                let cx = on_cx.clone();

                async move {
                    let date = resolve_date(date)?;

                    let holidays = cx
                        .query("holidays.on", || async {
                            cx.state.repos.holiday().between(date, date).await
                        })
                        .await?;

                    let regions = &cx.state.settings.holidays.regions;

                    Ok(holidays
                        .into_iter()
                        .find(|holiday| holiday.observed_in(regions))
                        .map(|holiday| holiday.name))
                }
            })
        })?;

        let in_week_cx = cx.clone();
        cx.expose(table, &IN_WEEK, || {
            lua.create_async_function(move |_, date: Option<String>| {
                let cx = in_week_cx.clone();

                async move {
                    let monday = resolve_date(date)?.week(Weekday::Mon).first_day();
                    let friday = monday + TimeDelta::days(4);

                    let holidays = cx
                        .query("holidays.in_week", || async {
                            cx.state.repos.holiday().between(monday, friday).await
                        })
                        .await?;

                    let regions = &cx.state.settings.holidays.regions;

                    Ok(holidays.iter().any(|holiday| holiday.observed_in(regions)))
                }
            })
        })
    }
}
