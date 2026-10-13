use chrono::{TimeDelta, Utc};
use mlua::ExternalError;

use crate::auth::scope::{Action, Scope};
use crate::lua::{LuaCallContext, LuaClass, lua_module};
use crate::repo::device_lock::DeviceLock;

use super::DeviceLocks;

#[derive(LuaClass)]
#[lua(output)]
pub struct DeviceLockStatus {
    source: String,
    locked_at: i64,
    expires_at: Option<i64>,
}

impl From<DeviceLock> for DeviceLockStatus {
    fn from(lock: DeviceLock) -> Self {
        Self {
            source: lock.source.to_string(),
            locked_at: lock.locked_at.timestamp(),
            expires_at: lock.expires_at.map(|expires_at| expires_at.timestamp()),
        }
    }
}

fn lockable_id(cx: &LuaCallContext, device: &str, action: Action) -> mlua::Result<String> {
    let devices = &cx.state.devices;
    let address = devices.address_or_self(device);

    let Some(lockable) = devices.lockable(address) else {
        return Err(format!("`{device}` can't be locked").into_lua_err());
    };

    let Some(id) = devices.id_for_address(address) else {
        return Err(format!("`{device}` is not a registered device").into_lua_err());
    };

    if !cx.allows(lockable.resource, action) {
        let scope = Scope::new(lockable.resource, action);

        tracing::info!(
            "[{}] lua scope check denied {scope} for {}",
            cx.event_id,
            cx.authority.describe()
        );

        return Err(format!("missing scope `{scope}`").into_lua_err());
    }

    Ok(id.to_owned())
}

pub struct DeviceLockLua;

#[lua_module(namespace = "device_lock")]
impl DeviceLockLua {
    #[lua]
    async fn state(cx: &LuaCallContext, device: String) -> mlua::Result<Option<DeviceLockStatus>> {
        let id = lockable_id(cx, &device, Action::Read)?;

        let lock = cx
            .query(Self::STATE, || async {
                cx.state.repos.device_lock().active(&id).await
            })
            .await?;

        Ok(lock.map(DeviceLockStatus::from))
    }

    #[lua]
    async fn lock(cx: &LuaCallContext, device: String, seconds: Option<i64>) -> mlua::Result<()> {
        let id = lockable_id(cx, &device, Action::Write)?;

        let expires_at = seconds.map(|seconds| Utc::now() + TimeDelta::seconds(seconds.max(0)));

        let detail = match seconds {
            Some(seconds) => format!("{device} for {seconds}s"),
            None => format!("{device} until unlocked"),
        };

        cx.command(Self::LOCK, detail, || async {
            DeviceLocks::new(&cx.state.repos, &cx.state.event_bus)
                .lock(&id, expires_at)
                .await
        })
        .await
    }

    #[lua]
    async fn unlock(cx: &LuaCallContext, device: String) -> mlua::Result<()> {
        let id = lockable_id(cx, &device, Action::Write)?;

        cx.command(Self::UNLOCK, &device, || async {
            DeviceLocks::new(&cx.state.repos, &cx.state.event_bus)
                .unlock(&id)
                .await
        })
        .await
    }
}
