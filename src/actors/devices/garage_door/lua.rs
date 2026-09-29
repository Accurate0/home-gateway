use mlua::{Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::actors::workflows::WorkflowWorker;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{
    LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType, schema,
};
use crate::settings::workflow::GarageDoorCommand;

const STATE: LuaClass = LuaClass {
    name: "GarageDoorStatus",
    fields: &[
        LuaField {
            name: "state",
            ty: LuaType::String,
        },
        LuaField {
            name: "open",
            ty: LuaType::Boolean,
        },
        LuaField {
            name: "contact",
            ty: LuaType::Optional(&LuaType::Boolean),
        },
        LuaField {
            name: "changed_at",
            ty: LuaType::Integer,
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
    scope: Some(Scope::new(Resource::GarageDoor, Action::Read)),
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
            ty: LuaType::Schema(schema::<GarageDoorCommand>),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::GarageDoor, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[GET, COMMAND];

pub struct GarageDoorLua;

impl LuaModule for GarageDoorLua {
    fn namespace(&self) -> &'static str {
        "garage_door"
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
                    let Some(id) = cx.state.devices.resolve_id(&device).map(str::to_owned) else {
                        return Ok(LuaValue::Nil);
                    };

                    let row = cx
                        .query("garage_door.state", || async {
                            cx.state.repos.garage_door().latest(&id).await
                        })
                        .await?;

                    let Some(row) = row else {
                        return Ok(LuaValue::Nil);
                    };

                    let result = lua.create_table()?;

                    result.set("state", row.state.to_string())?;
                    result.set("open", row.state != crate::db::GarageDoorState::Closed)?;
                    result.set("contact", row.contact)?;
                    result.set("changed_at", row.changed_at.timestamp())?;
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
                    let command: GarageDoorCommand = lua.from_value(command)?;
                    let worker = WorkflowWorker::new(cx.state.clone());

                    cx.command(
                        "garage_door.command",
                        format!("{device} {command}"),
                        || async { worker.run_garage_door(&device, command).await },
                    )
                    .await
                }
            })
        })
    }
}
