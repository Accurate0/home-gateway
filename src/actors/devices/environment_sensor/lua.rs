use mlua::{ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::actors::workflows::conditions::{environment_metric, query_environment};
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{
    LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType, schema,
};
use crate::settings::workflow::EnvMetric;

const READING: LuaClass = LuaClass {
    name: "EnvironmentReading",
    fields: &[
        LuaField {
            name: "temperature",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "humidity",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "pressure",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "lux",
            ty: LuaType::Optional(&LuaType::Number),
        },
        LuaField {
            name: "uv_index",
            ty: LuaType::Optional(&LuaType::Number),
        },
    ],
};

const GET: LuaFunction = LuaFunction {
    name: "get",
    params: &[
        LuaParam {
            name: "device",
            ty: LuaType::String,
        },
        LuaParam {
            name: "metric",
            ty: LuaType::Schema(schema::<EnvMetric>),
        },
    ],
    returns: Some(LuaType::Optional(&LuaType::Number)),
    scope: Some(Scope::new(Resource::Environment, Action::Read)),
};

const READING_FN: LuaFunction = LuaFunction {
    name: "reading",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Optional(&LuaType::Class(&READING))),
    scope: Some(Scope::new(Resource::Environment, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[GET, READING_FN];

pub struct EnvironmentLua;

impl LuaModule for EnvironmentLua {
    fn namespace(&self) -> &'static str {
        "environment"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let get_cx = cx.clone();
        cx.expose(table, &GET, || {
            lua.create_async_function(move |lua, (device, metric): (String, LuaValue)| {
                let cx = get_cx.clone();

                async move {
                    let metric: EnvMetric = lua.from_value(metric)?;
                    let sensor = cx.state.devices.address_or_self(&device).to_owned();

                    let reading = cx
                        .query("environment.get", || async {
                            query_environment(&sensor, cx.timeout()).await
                        })
                        .await?;

                    Ok(reading.and_then(|reading| environment_metric(&reading, metric)))
                }
            })
        })?;

        let reading_cx = cx.clone();
        cx.expose(table, &READING_FN, || {
            lua.create_async_function(move |lua, device: String| {
                let cx = reading_cx.clone();

                async move {
                    let sensor = cx.state.devices.address_or_self(&device).to_owned();

                    let reading = cx
                        .query("environment.reading", || async {
                            query_environment(&sensor, cx.timeout()).await
                        })
                        .await?;

                    let Some(reading) = reading else {
                        return Ok(LuaValue::Nil);
                    };

                    let table = lua.create_table().into_lua_err()?;
                    table.set("temperature", reading.temperature)?;
                    table.set("humidity", reading.humidity)?;
                    table.set("pressure", reading.pressure)?;
                    table.set("lux", reading.lux)?;
                    table.set("uv_index", reading.uv_index)?;

                    Ok(LuaValue::Table(table))
                }
            })
        })
    }
}
