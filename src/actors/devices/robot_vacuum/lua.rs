use mlua::{Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::actors::workflows::WorkflowWorker;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{
    LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType, schema,
};
use crate::settings::workflow::VacuumCommand;

const STATE: LuaClass = LuaClass {
    name: "VacuumState",
    fields: &[
        LuaField {
            name: "state",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "battery",
            ty: LuaType::Optional(&LuaType::Integer),
        },
        LuaField {
            name: "fan_speed",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "room",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "updated_at",
            ty: LuaType::Integer,
        },
    ],
};

const GET: LuaFunction = LuaFunction {
    name: "state",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Optional(&LuaType::Class(&STATE))),
    scope: Some(Scope::new(Resource::RobotVacuum, Action::Read)),
};

const COMMAND: LuaFunction = LuaFunction {
    name: "command",
    params: &[
        LuaParam {
            name: "device",
            ty: LuaType::String,
        },
        LuaParam {
            name: "command",
            ty: LuaType::Schema(schema::<VacuumCommand>),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::RobotVacuum, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[GET, COMMAND];

pub struct VacuumLua;

impl LuaModule for VacuumLua {
    fn namespace(&self) -> &'static str {
        "vacuum"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let state_cx = cx.clone();
        cx.expose(table, &GET, || {
            lua.create_async_function(move |lua, device: String| {
                let cx = state_cx.clone();

                async move {
                    let address = cx.state.devices.address_or_self(&device).to_owned();
                    let keys = vec![address];

                    let rows = cx
                        .query("vacuum.state", || async {
                            cx.state.repos.robot_vacuum().latest_many(&keys).await
                        })
                        .await?;

                    let Some(row) = rows.into_iter().max_by_key(|row| row.updated_at) else {
                        return Ok(LuaValue::Nil);
                    };

                    let result = lua.create_table()?;

                    result.set("state", row.state)?;
                    result.set("battery", row.battery_level)?;
                    result.set("fan_speed", row.fan_speed)?;
                    result.set("room", row.room)?;
                    result.set("updated_at", row.updated_at.timestamp())?;

                    Ok(LuaValue::Table(result))
                }
            })
        })?;

        let command_cx = cx.clone();
        cx.expose(table, &COMMAND, || {
            lua.create_async_function(move |lua, (device, command): (String, LuaValue)| {
                let cx = command_cx.clone();

                async move {
                    let command: VacuumCommand = lua.from_value(command)?;
                    let worker = WorkflowWorker::new(cx.state.clone());

                    cx.command(
                        "vacuum.command",
                        format!("{device} {command:?}"),
                        || async { worker.run_robot_vacuum(&device, command).await },
                    )
                    .await
                }
            })
        })
    }
}
