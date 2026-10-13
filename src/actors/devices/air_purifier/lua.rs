use crate::actors::workflows::WorkflowWorker;
use crate::lua::{Json, LuaCallContext, LuaClass, lua_module};
use crate::repo::device_lock::DeviceLock;
use crate::workflows::WorkflowError;
use crate::workflows::definition::AirPurifierCommand;

#[derive(LuaClass)]
#[lua(output)]
pub struct AirPurifierStatus {
    on: bool,
    mode: Option<String>,
    speed: Option<i32>,
    pm25: Option<f64>,
    filter_life: Option<f64>,
    display: Option<bool>,
    changed_at: i64,
    updated_at: i64,
    locked: bool,
    lock_source: Option<String>,
    locked_until: Option<i64>,
}

pub struct AirPurifierLua;

#[lua_module(namespace = "air_purifier")]
impl AirPurifierLua {
    #[lua(scope = AirPurifier::Read)]
    async fn state(cx: &LuaCallContext, device: String) -> mlua::Result<Option<AirPurifierStatus>> {
        let Some(id) = cx.state.devices.resolve_id(&device).map(str::to_owned) else {
            return Ok(None);
        };

        let (row, lock) = cx
            .query(Self::STATE, || async {
                let row = cx.state.repos.air_purifier().latest(&id).await?;
                let lock = cx.state.repos.device_lock().active(&id).await?;

                Ok::<_, sqlx::Error>((row, lock))
            })
            .await?;

        Ok(row.map(|row| AirPurifierStatus {
            on: row.is_on,
            mode: row.mode.map(|mode| mode.to_string()),
            speed: row.speed,
            pm25: row.pm25,
            filter_life: row.filter_life,
            display: row.display,
            changed_at: row.changed_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
            locked: lock.is_some(),
            lock_source: lock.as_ref().map(|lock| lock.source.to_string()),
            locked_until: lock
                .as_ref()
                .and_then(|lock: &DeviceLock| lock.expires_at)
                .map(|expires_at| expires_at.timestamp()),
        }))
    }

    #[lua(scope = AirPurifier::Write)]
    async fn command(
        cx: &LuaCallContext,
        device: String,
        command: Json<AirPurifierCommand>,
    ) -> mlua::Result<()> {
        let command = command.0;
        let worker = WorkflowWorker::new(cx.state.clone());

        cx.command(Self::COMMAND, format!("{device} {command}"), || async {
            let outcome = worker.run_air_purifier(&device, command).await?;

            WorkflowWorker::warn_if_locked(&cx.trace, &device, outcome);

            Ok::<_, WorkflowError>(outcome)
        })
        .await
    }
}
