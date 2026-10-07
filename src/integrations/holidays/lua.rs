use chrono::{NaiveDate, TimeDelta, Utc, Weekday};
use chrono_tz::Australia::Perth;
use mlua::ExternalResult;

use crate::lua::{LuaCallContext, lua_module};

pub struct HolidaysLua;

fn resolve_date(date: Option<String>) -> mlua::Result<NaiveDate> {
    match date {
        Some(date) => NaiveDate::parse_from_str(&date, "%Y-%m-%d").into_lua_err(),
        None => Ok(Utc::now().with_timezone(&Perth).date_naive()),
    }
}

#[lua_module(namespace = "holidays")]
impl HolidaysLua {
    #[lua(scope = Holiday::Read)]
    async fn on(cx: &LuaCallContext, date: Option<String>) -> mlua::Result<Option<String>> {
        let date = resolve_date(date)?;

        let holidays = cx
            .query(Self::ON, || async {
                cx.state.repos.holiday().between(date, date).await
            })
            .await?;

        let regions = &cx.state.settings.integrations.holidays.regions;

        Ok(holidays
            .into_iter()
            .find(|holiday| holiday.observed_in(regions))
            .map(|holiday| holiday.name))
    }

    #[lua(scope = Holiday::Read)]
    async fn in_week(cx: &LuaCallContext, date: Option<String>) -> mlua::Result<bool> {
        let monday = resolve_date(date)?.week(Weekday::Mon).first_day();
        let friday = monday + TimeDelta::days(4);

        let holidays = cx
            .query(Self::IN_WEEK, || async {
                cx.state.repos.holiday().between(monday, friday).await
            })
            .await?;

        let regions = &cx.state.settings.integrations.holidays.regions;

        Ok(holidays.iter().any(|holiday| holiday.observed_in(regions)))
    }
}
