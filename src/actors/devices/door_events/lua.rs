use mlua::{Lua, Table};

use crate::actors::workflows::conditions::query_door_open;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const IS_OPEN: LuaFunction = LuaFunction {
    name: "is_open",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Door, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[IS_OPEN];

pub struct DoorLua;

impl LuaModule for DoorLua {
    fn namespace(&self) -> &'static str {
        "door"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let open_cx = cx.clone();

        cx.expose(table, &IS_OPEN, || {
            lua.create_async_function(move |_, device: String| {
                let cx = open_cx.clone();

                async move {
                    let ieee_addr = cx.state.devices.address_or_self(&device).to_owned();

                    cx.query("door.is_open", || async {
                        query_door_open(&ieee_addr, cx.timeout()).await
                    })
                    .await
                }
            })
        })
    }
}
