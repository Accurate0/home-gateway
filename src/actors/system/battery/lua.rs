use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType};

const LOW_BATTERY: LuaClass = LuaClass {
    name: "LowBattery",
    fields: &[
        LuaField {
            name: "device",
            ty: LuaType::String,
        },
        LuaField {
            name: "level",
            ty: LuaType::Number,
        },
    ],
};

const LEVEL: LuaFunction = LuaFunction {
    name: "level",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Optional(&LuaType::Number)),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const LOW: LuaFunction = LuaFunction {
    name: "low",
    params: &[LuaParam {
        name: "threshold",
        ty: LuaType::Number,
    }],
    returns: Some(LuaType::Array(&LuaType::Class(&LOW_BATTERY))),
    scope: Some(Scope::new(Resource::Device, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[LEVEL, LOW];

pub struct BatteryLua;

impl LuaModule for BatteryLua {
    fn namespace(&self) -> &'static str {
        "battery"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let level_cx = cx.clone();
        cx.expose(table, &LEVEL, || {
            lua.create_async_function(move |_, device: String| {
                let cx = level_cx.clone();

                async move {
                    let keys = vec![cx.state.devices.address_or_self(&device).to_owned()];

                    let rows = cx
                        .query("battery.level", || async {
                            cx.state.repos.battery().latest_many(&keys).await
                        })
                        .await?;

                    Ok(rows.into_iter().find_map(|row| row.battery_percent))
                }
            })
        })?;

        let low_cx = cx.clone();
        cx.expose(table, &LOW, || {
            lua.create_async_function(move |lua, threshold: f64| {
                let cx = low_cx.clone();

                async move {
                    let keys: Vec<String> = cx
                        .state
                        .devices
                        .aliases()
                        .values()
                        .map(|address| address.to_string())
                        .collect();

                    let rows = cx
                        .query("battery.low", || async {
                            cx.state.repos.battery().latest_many(&keys).await
                        })
                        .await?;

                    let result = lua.create_table()?;

                    for row in rows {
                        let Some(level) = row.battery_percent.filter(|level| *level < threshold)
                        else {
                            continue;
                        };

                        let device = cx
                            .state
                            .devices
                            .id_for_address(&row.device_id)
                            .unwrap_or(&row.device_id);

                        let entry = lua.create_table()?;
                        entry.set("device", device)?;
                        entry.set("level", level)?;

                        result.push(entry)?;
                    }

                    Ok(result)
                }
            })
        })
    }
}
