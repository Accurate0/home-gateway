use crate::actors::workflows::WorkflowWorker;
use crate::lua::{Json, LuaCallContext, LuaClass, lua_module};
use crate::workflows::definition::VacuumCommand;

#[derive(LuaClass)]
#[lua(output)]
pub struct VacuumState {
    state: Option<String>,
    battery: Option<i32>,
    fan_speed: Option<String>,
    room: Option<String>,
    updated_at: i64,
}

pub struct VacuumLua;

#[lua_module(namespace = "vacuum")]
impl VacuumLua {
    #[lua(scope = RobotVacuum::Read)]
    async fn state(cx: &LuaCallContext, device: String) -> mlua::Result<Option<VacuumState>> {
        let address = cx.state.devices.address_or_self(&device).to_owned();
        let keys = vec![address];

        let rows = cx
            .query(Self::STATE, || async {
                cx.state.repos.robot_vacuum().latest_many(&keys).await
            })
            .await?;

        let latest = rows.into_iter().max_by_key(|row| row.updated_at);

        Ok(latest.map(|row| VacuumState {
            state: row.state,
            battery: row.battery_level,
            fan_speed: row.fan_speed,
            room: row.room,
            updated_at: row.updated_at.timestamp(),
        }))
    }

    #[lua(scope = RobotVacuum::Write)]
    async fn command(
        cx: &LuaCallContext,
        device: String,
        command: Json<VacuumCommand>,
    ) -> mlua::Result<()> {
        let command = command.0;
        let worker = WorkflowWorker::new(cx.state.clone());

        cx.command(Self::COMMAND, format!("{device} {command:?}"), || async {
            worker.run_robot_vacuum(&device, command).await
        })
        .await
    }
}
