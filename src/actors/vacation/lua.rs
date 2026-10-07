use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::lua::{LuaCallContext, lua_module};

use super::{VacationActor, VacationMessage};

pub struct VacationLua;

#[lua_module(namespace = "vacation")]
impl VacationLua {
    #[lua(scope = Vacation::Read)]
    async fn active(cx: &LuaCallContext) -> mlua::Result<bool> {
        let mode = cx
            .state
            .handles
            .expect::<WorkflowManager>()
            .current_mode()
            .await;

        Ok(cx.state.settings.vacation.arms_on(&mode))
    }

    #[lua(scope = Vacation::Write)]
    async fn arm(cx: &LuaCallContext, armed: bool) -> mlua::Result<()> {
        cx.command(Self::ARM, armed, || async {
            rpc::cast(VacationActor::NAME, VacationMessage::Arm(armed))
        })
        .await
    }
}
