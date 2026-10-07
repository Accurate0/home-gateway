use crate::actors::workflows::WorkflowWorker;
use crate::db::GarageDoorState;
use crate::lua::{Json, LuaCallContext, LuaClass, lua_module};
use crate::settings::workflow::GarageDoorCommand;

#[derive(LuaClass)]
#[lua(output)]
pub struct GarageDoorStatus {
    state: String,
    open: bool,
    contact: Option<bool>,
    changed_at: i64,
    updated_at: i64,
}

pub struct GarageDoorLua;

#[lua_module(namespace = "garage_door")]
impl GarageDoorLua {
    #[lua(scope = GarageDoor::Read)]
    async fn state(cx: &LuaCallContext, device: String) -> mlua::Result<Option<GarageDoorStatus>> {
        let Some(id) = cx.state.devices.resolve_id(&device).map(str::to_owned) else {
            return Ok(None);
        };

        let row = cx
            .query(Self::STATE, || async {
                cx.state.repos.garage_door().latest(&id).await
            })
            .await?;

        Ok(row.map(|row| GarageDoorStatus {
            state: row.state.to_string(),
            open: row.state != GarageDoorState::Closed,
            contact: row.contact,
            changed_at: row.changed_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
        }))
    }

    #[lua(scope = GarageDoor::Write)]
    async fn command(
        cx: &LuaCallContext,
        device: String,
        command: Json<GarageDoorCommand>,
    ) -> mlua::Result<()> {
        let command = command.0;
        let worker = WorkflowWorker::new(cx.state.clone());

        cx.command(Self::COMMAND, format!("{device} {command}"), || async {
            worker.run_garage_door(&device, command).await
        })
        .await
    }
}
