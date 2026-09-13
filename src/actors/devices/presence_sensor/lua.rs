use mlua::{Lua, Table};

use crate::actors::workflows::conditions::query_presence;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

const AT: LuaFunction = LuaFunction {
    name: "at",
    params: &[LuaParam {
        name: "device",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Presence, Action::Read)),
};

const FUNCTIONS: &[LuaFunction] = &[AT];

pub struct PresenceLua;

impl LuaModule for PresenceLua {
    fn namespace(&self) -> &'static str {
        "presence"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let at_cx = cx.clone();

        cx.expose(table, &AT, || {
            lua.create_async_function(move |_, device: String| {
                let cx = at_cx.clone();

                async move {
                    let sensor = cx.state.devices.address_or_self(&device).to_owned();

                    cx.query("presence.at", || async {
                        query_presence(&sensor, cx.timeout()).await
                    })
                    .await
                }
            })
        })
    }
}
