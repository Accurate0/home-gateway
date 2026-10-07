use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Australia::Perth;
use chrono_tz::Tz;
use mlua::{ExternalError, ExternalResult};

use super::{LuaClass, lua_module};

#[derive(LuaClass)]
#[lua(name = "TimeNow", output)]
pub struct Now {
    iso: String,
    epoch: i64,
    date: String,
    hour: u32,
    minute: u32,
    weekday: String,
}

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

#[lua_module(namespace = "time")]
impl TimeLua {
    #[lua]
    fn now() -> mlua::Result<Now> {
        let now = local_now();

        Ok(Now {
            iso: now.to_rfc3339(),
            epoch: now.timestamp(),
            date: now.format("%Y-%m-%d").to_string(),
            hour: now.hour(),
            minute: now.minute(),
            weekday: now.weekday().to_string(),
        })
    }

    #[lua]
    fn between(start: String, finish: String) -> mlua::Result<bool> {
        Ok(within(local_now().time(), clock(&start)?, clock(&finish)?))
    }

    #[lua]
    fn weekend() -> mlua::Result<bool> {
        Ok(matches!(local_now().weekday(), Weekday::Sat | Weekday::Sun))
    }

    #[lua]
    fn parse(iso: String) -> mlua::Result<i64> {
        Ok(DateTime::parse_from_rfc3339(&iso)
            .into_lua_err()?
            .timestamp())
    }

    #[lua]
    fn format(epoch: i64, pattern: String) -> mlua::Result<String> {
        let moment = Perth
            .timestamp_opt(epoch, 0)
            .single()
            .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

        Ok(moment.format(&pattern).to_string())
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
