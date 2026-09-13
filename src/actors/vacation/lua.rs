use mlua::{Lua, Table};

use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

use super::{VacationActor, VacationMessage};

const ACTIVE: LuaFunction = LuaFunction {
    name: "active",
    params: &[],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Vacation, Action::Read)),
};

const ARM: LuaFunction = LuaFunction {
    name: "arm",
    params: &[LuaParam {
        name: "armed",
        ty: LuaType::Boolean,
    }],
    returns: None,
    scope: Some(Scope::new(Resource::Vacation, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[ACTIVE, ARM];

pub struct VacationLua;

impl LuaModule for VacationLua {
    fn namespace(&self) -> &'static str {
        "vacation"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let active_cx = cx.clone();
        cx.expose(table, &ACTIVE, || {
            lua.create_async_function(move |_, ()| {
                let cx = active_cx.clone();

                async move {
                    let mode = cx
                        .state
                        .handles
                        .expect::<WorkflowManager>()
                        .current_mode()
                        .await;

                    Ok(cx.state.settings.vacation.arms_on(&mode))
                }
            })
        })?;

        let arm_cx = cx.clone();
        cx.expose(table, &ARM, || {
            lua.create_async_function(move |_, armed: bool| {
                let cx = arm_cx.clone();

                async move {
                    cx.command("vacation.arm", armed, || async {
                        rpc::cast(VacationActor::NAME, VacationMessage::Arm(armed))
                    })
                    .await
                }
            })
        })
    }
}
