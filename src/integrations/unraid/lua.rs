use crate::lua::{LuaCallContext, LuaClass, lua_module};
use crate::repo::unraid::StoredUnraidSnapshot;

use super::types::{ArraySnapshot, DiskSnapshot};

#[derive(LuaClass)]
#[lua(output)]
pub struct UnraidArrayInfo {
    state: String,
    usage_percent: Option<f64>,
    kilobytes_free: i64,
    kilobytes_used: i64,
    kilobytes_total: i64,
    parity_status: String,
    parity_progress: Option<i64>,
    parity_errors: Option<i64>,
    parity_running: bool,
    recorded_at: i64,
}

#[derive(LuaClass)]
#[lua(output)]
pub struct UnraidDiskInfo {
    name: String,
    kind: String,
    status: Option<String>,
    temperature: Option<i32>,
    usage_percent: Option<f64>,
    fs_used_kb: Option<i64>,
    fs_size_kb: Option<i64>,
    errors: Option<i64>,
    spinning: Option<bool>,
}

impl UnraidArrayInfo {
    fn new(array: &ArraySnapshot, recorded_at: i64) -> Self {
        Self {
            state: array.state.to_string(),
            usage_percent: array.usage_percent(),
            kilobytes_free: array.kilobytes_free,
            kilobytes_used: array.kilobytes_used,
            kilobytes_total: array.kilobytes_total,
            parity_status: array.parity_status.to_string(),
            parity_progress: array.parity_progress,
            parity_errors: array.parity_errors,
            parity_running: array.parity_running,
            recorded_at,
        }
    }
}

impl From<&DiskSnapshot> for UnraidDiskInfo {
    fn from(disk: &DiskSnapshot) -> Self {
        Self {
            name: disk.name.clone(),
            kind: disk.kind.to_string(),
            status: disk.status.map(|status| status.to_string()),
            temperature: disk.temperature,
            usage_percent: disk.usage_percent(),
            fs_used_kb: disk.fs_used_kb,
            fs_size_kb: disk.fs_size_kb,
            errors: disk.errors,
            spinning: disk.spinning,
        }
    }
}

pub struct UnraidLua;

impl UnraidLua {
    async fn latest(
        cx: &LuaCallContext,
        name: &'static str,
    ) -> mlua::Result<Option<StoredUnraidSnapshot>> {
        cx.query(name, || async { cx.state.repos.unraid().latest().await })
            .await
    }
}

#[lua_module(namespace = "unraid")]
impl UnraidLua {
    #[lua(scope = Unraid::Read)]
    async fn array(cx: &LuaCallContext) -> mlua::Result<Option<UnraidArrayInfo>> {
        Ok(Self::latest(cx, Self::ARRAY).await?.map(|stored| {
            UnraidArrayInfo::new(&stored.snapshot.array, stored.recorded_at.timestamp())
        }))
    }

    #[lua(scope = Unraid::Read)]
    async fn disks(cx: &LuaCallContext) -> mlua::Result<Vec<UnraidDiskInfo>> {
        Ok(Self::latest(cx, Self::DISKS)
            .await?
            .map(|stored| stored.snapshot.disks.iter().map(UnraidDiskInfo::from).collect())
            .unwrap_or_default())
    }

    #[lua(scope = Unraid::Read)]
    async fn disk(cx: &LuaCallContext, name: String) -> mlua::Result<Option<UnraidDiskInfo>> {
        Ok(Self::latest(cx, Self::DISK).await?.and_then(|stored| {
            stored
                .snapshot
                .disks
                .iter()
                .find(|disk| disk.name == name)
                .map(UnraidDiskInfo::from)
        }))
    }
}
