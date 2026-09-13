use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Australia::Perth;
use chrono_tz::Tz;
use mlua::{ExternalError, ExternalResult, Lua, Table};

use super::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const NOW_CLASS: LuaClass = LuaClass {
    name: "TimeNow",
    fields: &[
        LuaField {
            name: "iso",
            ty: LuaType::String,
        },
        LuaField {
            name: "epoch",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "date",
            ty: LuaType::String,
        },
        LuaField {
            name: "hour",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "minute",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "weekday",
            ty: LuaType::String,
        },
    ],
};

const NOW: LuaFunction = LuaFunction {
    name: "now",
    params: &[],
    returns: Some(LuaType::Class(&NOW_CLASS)),
    scope: None,
};

const BETWEEN: LuaFunction = LuaFunction {
    name: "between",
    params: &[
        LuaParam {
            name: "start",
            ty: LuaType::String,
        },
        LuaParam {
            name: "finish",
            ty: LuaType::String,
        },
    ],
    returns: Some(LuaType::Boolean),
    scope: None,
};

const WEEKEND: LuaFunction = LuaFunction {
    name: "weekend",
    params: &[],
    returns: Some(LuaType::Boolean),
    scope: None,
};

const PARSE: LuaFunction = LuaFunction {
    name: "parse",
    params: &[LuaParam {
        name: "iso",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Integer),
    scope: None,
};

const FORMAT: LuaFunction = LuaFunction {
    name: "format",
    params: &[
        LuaParam {
            name: "epoch",
            ty: LuaType::Integer,
        },
        LuaParam {
            name: "pattern",
            ty: LuaType::String,
        },
    ],
    returns: Some(LuaType::String),
    scope: None,
};

const FUNCTIONS: &[LuaFunction] = &[NOW, BETWEEN, WEEKEND, PARSE, FORMAT];

pub struct TimeLua;

fn local_now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Perth)
}

fn clock(raw: &str) -> mlua::Result<NaiveTime> {
    NaiveTime::parse_from_str(raw, "%H:%M").into_lua_err()
}

fn within(now: NaiveTime, start: NaiveTime, finish: NaiveTime) -> bool {
    if start <= finish {
        now >= start && now < finish
    } else {
        now >= start || now < finish
    }
}

impl LuaModule for TimeLua {
    fn namespace(&self) -> &'static str {
        "time"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        cx.expose(table, &NOW, || {
            lua.create_function(|lua, ()| {
                let now = local_now();
                let result = lua.create_table()?;

                result.set("iso", now.to_rfc3339())?;
                result.set("epoch", now.timestamp())?;
                result.set("date", now.format("%Y-%m-%d").to_string())?;
                result.set("hour", now.hour())?;
                result.set("minute", now.minute())?;
                result.set("weekday", now.weekday().to_string())?;

                Ok(result)
            })
        })?;

        cx.expose(table, &BETWEEN, || {
            lua.create_function(|_, (start, finish): (String, String)| {
                Ok(within(local_now().time(), clock(&start)?, clock(&finish)?))
            })
        })?;

        cx.expose(table, &WEEKEND, || {
            lua.create_function(|_, ()| {
                Ok(matches!(local_now().weekday(), Weekday::Sat | Weekday::Sun))
            })
        })?;

        cx.expose(table, &PARSE, || {
            lua.create_function(|_, iso: String| {
                Ok(DateTime::parse_from_rfc3339(&iso)
                    .into_lua_err()?
                    .timestamp())
            })
        })?;

        cx.expose(table, &FORMAT, || {
            lua.create_function(|_, (epoch, pattern): (i64, String)| {
                let moment = Perth
                    .timestamp_opt(epoch, 0)
                    .single()
                    .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

                Ok(moment.format(&pattern).to_string())
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::within;

    fn at(raw: &str) -> NaiveTime {
        NaiveTime::parse_from_str(raw, "%H:%M").expect("expected a clock time")
    }

    #[test]
    fn a_daytime_window_contains_only_its_span() {
        assert!(within(at("12:00"), at("09:00"), at("17:00")));
        assert!(!within(at("08:59"), at("09:00"), at("17:00")));
        assert!(!within(at("17:00"), at("09:00"), at("17:00")));
    }

    #[test]
    fn an_overnight_window_wraps_past_midnight() {
        assert!(within(at("23:30"), at("22:00"), at("06:00")));
        assert!(within(at("02:00"), at("22:00"), at("06:00")));
        assert!(!within(at("12:00"), at("22:00"), at("06:00")));
    }
}
