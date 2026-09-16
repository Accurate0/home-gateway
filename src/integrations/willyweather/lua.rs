use mlua::{Lua, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

use super::types::{Forecast, ForecastDetails, ForecastHour};

const DAY: LuaClass = LuaClass {
    name: "ForecastDay",
    fields: &[
        LuaField {
            name: "date_time",
            ty: LuaType::String,
        },
        LuaField {
            name: "code",
            ty: LuaType::String,
        },
        LuaField {
            name: "description",
            ty: LuaType::String,
        },
        LuaField {
            name: "emoji",
            ty: LuaType::String,
        },
        LuaField {
            name: "min",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "max",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "uv",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "rain_probability",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaField {
            name: "rain_start_range",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaField {
            name: "rain_end_range",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaField {
            name: "rain_range_code",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "wind_max_speed",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "first_light",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "sunrise",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "sunset",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "last_light",
            ty: LuaType::Optional(&LuaType::String),
        },
    ],
};

const HOUR: LuaClass = LuaClass {
    name: "ForecastHour",
    fields: &[
        LuaField {
            name: "date_time",
            ty: LuaType::String,
        },
        LuaField {
            name: "temperature",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "wind_speed",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "wind_direction",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "wind_direction_text",
            ty: LuaType::Optional(&LuaType::String),
        },
    ],
};

const FORECAST: LuaClass = LuaClass {
    name: "Forecast",
    fields: &[
        LuaField {
            name: "days",
            ty: LuaType::Array(&LuaType::Class(&DAY)),
        },
        LuaField {
            name: "hours",
            ty: LuaType::Array(&LuaType::Class(&HOUR)),
        },
    ],
};

const LOCATION: LuaParam = LuaParam {
    name: "location",
    ty: LuaType::Optional(&LuaType::String),
};

const FORECAST_FN: LuaFunction = LuaFunction {
    name: "forecast",
    params: &[LOCATION],
    returns: Some(LuaType::Optional(&LuaType::Class(&FORECAST))),
    scope: Some(Scope::new(Resource::Weather, Action::Read)),
};

const TODAY: LuaFunction = LuaFunction {
    name: "today",
    params: &[LOCATION],
    returns: Some(LuaType::Optional(&LuaType::Class(&DAY))),
    scope: Some(Scope::new(Resource::Weather, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[FORECAST_FN, TODAY];

pub struct WeatherLua;

impl LuaModule for WeatherLua {
    fn namespace(&self) -> &'static str {
        "weather"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let forecast_cx = cx.clone();
        cx.expose(table, &FORECAST_FN, || {
            lua.create_async_function(move |lua, location: Option<String>| {
                let cx = forecast_cx.clone();

                async move {
                    let Some(forecast) = fetch(&cx, location.as_deref()).await? else {
                        return Ok(LuaValue::Nil);
                    };

                    Ok(LuaValue::Table(forecast_table(&lua, &forecast)?))
                }
            })
        })?;

        let today_cx = cx.clone();
        cx.expose(table, &TODAY, || {
            lua.create_async_function(move |lua, location: Option<String>| {
                let cx = today_cx.clone();

                async move {
                    let Some(forecast) = fetch(&cx, location.as_deref()).await? else {
                        return Ok(LuaValue::Nil);
                    };

                    let Some(today) = forecast.days.first() else {
                        return Ok(LuaValue::Nil);
                    };

                    Ok(LuaValue::Table(day_table(&lua, today)?))
                }
            })
        })
    }
}

async fn fetch(cx: &LuaCallContext, location: Option<&str>) -> mlua::Result<Option<Forecast>> {
    let settings = &cx.state.settings.willyweather;
    let requested = location.unwrap_or(&settings.default_location);

    let Some(alias) = settings.resolve_location(requested) else {
        return Err(mlua::Error::external(format!(
            "unknown weather location `{requested}`; available: [{}]",
            settings
                .locations
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )));
    };

    let alias = alias.to_owned();

    let forecast = cx
        .query("weather.forecast", || async {
            cx.state.repos.willyweather().forecast(&alias).await
        })
        .await?;

    if forecast.is_none() {
        tracing::info!(
            "[{}] lua weather has no stored forecast for {alias}",
            cx.event_id
        );
    }

    Ok(forecast)
}

fn forecast_table(lua: &Lua, forecast: &Forecast) -> mlua::Result<Table> {
    let days = lua.create_table()?;
    for day in &forecast.days {
        days.push(day_table(lua, day)?)?;
    }

    let hours = lua.create_table()?;
    for hour in &forecast.hours {
        hours.push(hour_table(lua, hour)?)?;
    }

    let table = lua.create_table()?;
    table.set("days", days)?;
    table.set("hours", hours)?;

    Ok(table)
}

fn day_table(lua: &Lua, day: &ForecastDetails) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set("date_time", day.date_time.as_str())?;
    table.set("code", day.code.as_str())?;
    table.set("description", day.description.as_str())?;
    table.set("emoji", day.emoji.as_str())?;
    table.set("min", day.min)?;
    table.set("max", day.max)?;
    table.set("uv", day.uv)?;
    table.set("rain_probability", day.rain_probability)?;
    table.set("rain_start_range", day.rain_start_range)?;
    table.set("rain_end_range", day.rain_end_range)?;
    table.set("rain_range_code", day.rain_range_code.as_deref())?;
    table.set("wind_max_speed", day.wind_max_speed)?;
    table.set("first_light", day.first_light.as_deref())?;
    table.set("sunrise", day.sunrise.as_deref())?;
    table.set("sunset", day.sunset.as_deref())?;
    table.set("last_light", day.last_light.as_deref())?;

    Ok(table)
}

fn hour_table(lua: &Lua, hour: &ForecastHour) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set("date_time", hour.date_time.as_str())?;
    table.set("temperature", hour.temperature)?;
    table.set("wind_speed", hour.wind_speed)?;
    table.set("wind_direction", hour.wind_direction)?;
    table.set("wind_direction_text", hour.wind_direction_text.as_deref())?;

    Ok(table)
}
