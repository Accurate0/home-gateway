use chrono::{DateTime, TimeDelta, Utc};
use mlua::{ExternalError, Lua, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::device_registry::last_seen;
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};
use crate::repo::metric::MetricRow;

const DEVICE: LuaParam = LuaParam {
    name: "device",
    ty: LuaType::String,
};

const KEY: LuaParam = LuaParam {
    name: "key",
    ty: LuaType::String,
};

const SAMPLE: LuaClass = LuaClass {
    name: "MetricSample",
    fields: &[
        LuaField {
            name: "value",
            ty: LuaType::Any,
        },
        LuaField {
            name: "at",
            ty: LuaType::Integer,
        },
    ],
};

const LAST_SEEN: LuaFunction = LuaFunction {
    name: "last_seen",
    params: &[DEVICE],
    returns: Some(LuaType::Optional(&LuaType::Integer)),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const OFFLINE: LuaFunction = LuaFunction {
    name: "offline",
    params: &[LuaParam {
        name: "minutes",
        ty: LuaType::Integer,
    }],
    returns: Some(LuaType::Array(&LuaType::String)),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const METRIC: LuaFunction = LuaFunction {
    name: "metric",
    params: &[DEVICE, KEY],
    returns: Some(LuaType::Optional(&LuaType::Class(&SAMPLE))),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const METRIC_SINCE: LuaFunction = LuaFunction {
    name: "metric_since",
    params: &[
        DEVICE,
        KEY,
        LuaParam {
            name: "epoch",
            ty: LuaType::Integer,
        },
    ],
    returns: Some(LuaType::Array(&LuaType::Class(&SAMPLE))),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[LAST_SEEN, OFFLINE, METRIC, METRIC_SINCE];

pub struct DeviceLua;

fn sample(lua: &Lua, row: MetricRow) -> mlua::Result<Table> {
    let entry = lua.create_table()?;

    let value = match (row.value, row.text_value) {
        (Some(number), _) => LuaValue::Number(number),
        (None, Some(text)) => LuaValue::String(lua.create_string(text)?),
        (None, None) => LuaValue::Nil,
    };

    entry.set("value", value)?;
    entry.set("at", row.time.timestamp())?;

    Ok(entry)
}

impl LuaModule for DeviceLua {
    fn namespace(&self) -> &'static str {
        "device"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let last_seen_cx = cx.clone();
        cx.expose(table, &LAST_SEEN, || {
            lua.create_async_function(move |_, device: String| {
                let cx = last_seen_cx.clone();

                async move {
                    let keys = vec![cx.state.devices.address_or_self(&device).to_owned()];

                    let found = cx
                        .query("device.last_seen", || async {
                            last_seen::lookup(&cx.state.devices, cx.state.repos.device(), &keys)
                                .await
                        })
                        .await?;

                    Ok(found.values().map(|at| at.timestamp()).max())
                }
            })
        })?;

        let offline_cx = cx.clone();
        cx.expose(table, &OFFLINE, || {
            lua.create_async_function(move |_, minutes: i64| {
                let cx = offline_cx.clone();

                async move {
                    let aliases = cx.state.devices.aliases();
                    let keys: Vec<String> = aliases.values().map(|a| a.to_string()).collect();

                    let found = cx
                        .query("device.offline", || async {
                            last_seen::lookup(&cx.state.devices, cx.state.repos.device(), &keys)
                                .await
                        })
                        .await?;

                    let cutoff = Utc::now() - TimeDelta::minutes(minutes.max(0));

                    let mut offline: Vec<String> = aliases
                        .iter()
                        .filter(|(_, address)| {
                            found
                                .get(address.as_str())
                                .is_none_or(|last_seen| *last_seen < cutoff)
                        })
                        .map(|(id, _)| id.to_string())
                        .collect();

                    offline.sort();

                    Ok(offline)
                }
            })
        })?;

        let metric_cx = cx.clone();
        cx.expose(table, &METRIC, || {
            lua.create_async_function(move |lua, (device, key): (String, String)| {
                let cx = metric_cx.clone();

                async move {
                    let address = cx.state.devices.address_or_self(&device).to_owned();

                    let row = cx
                        .query("device.metric", || async {
                            cx.state.repos.metric().latest(&address, &key).await
                        })
                        .await?;

                    match row {
                        Some(row) => Ok(LuaValue::Table(sample(&lua, row)?)),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })
        })?;

        let since_cx = cx.clone();
        cx.expose(table, &METRIC_SINCE, || {
            lua.create_async_function(move |lua, (device, key, epoch): (String, String, i64)| {
                let cx = since_cx.clone();

                async move {
                    let address = cx.state.devices.address_or_self(&device).to_owned();
                    let earliest = DateTime::from_timestamp(epoch, 0)
                        .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

                    let rows = cx
                        .query("device.metric_since", || async {
                            cx.state
                                .repos
                                .metric()
                                .since(&address, &key, earliest)
                                .await
                        })
                        .await?;

                    let result = lua.create_table()?;

                    for row in rows {
                        result.push(sample(&lua, row)?)?;
                    }

                    Ok(result)
                }
            })
        })
    }
}
