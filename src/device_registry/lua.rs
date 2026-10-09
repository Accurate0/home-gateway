use chrono::{DateTime, TimeDelta, Utc};
use mlua::{ExternalError, Lua, Value as LuaValue};

use crate::device_registry::last_seen::LastSeen;
use crate::lua::{LuaCallContext, LuaClass, lua_module};
use crate::repo::metric::MetricRow;

#[derive(LuaClass)]
#[lua(output)]
pub struct MetricSample {
    value: LuaValue,
    at: i64,
}

impl MetricSample {
    fn new(lua: &Lua, row: MetricRow) -> mlua::Result<Self> {
        let value = match (row.value, row.text_value) {
            (Some(number), _) => LuaValue::Number(number),
            (None, Some(text)) => LuaValue::String(lua.create_string(text)?),
            (None, None) => LuaValue::Nil,
        };

        Ok(MetricSample {
            value,
            at: row.time.timestamp(),
        })
    }
}

pub struct DeviceLua;

#[lua_module(namespace = "device")]
impl DeviceLua {
    #[lua(scope = Device::Read)]
    async fn last_seen(cx: &LuaCallContext, device: String) -> mlua::Result<Option<i64>> {
        let keys = vec![cx.state.devices.address_or_self(&device).to_owned()];

        let found = cx
            .query(Self::LAST_SEEN, || async {
                cx.state.handles.expect::<LastSeen>().lookup(&keys).await
            })
            .await?;

        Ok(found.values().map(|at| at.timestamp()).max())
    }

    #[lua(scope = Device::Read)]
    async fn offline(cx: &LuaCallContext, minutes: i64) -> mlua::Result<Vec<String>> {
        let ids = cx.state.devices.ids();
        let keys: Vec<String> = ids.values().map(|a| a.to_string()).collect();

        let found = cx
            .query(Self::OFFLINE, || async {
                cx.state.handles.expect::<LastSeen>().lookup(&keys).await
            })
            .await?;

        let cutoff = Utc::now() - TimeDelta::minutes(minutes.max(0));

        let mut offline: Vec<String> = ids
            .iter()
            .filter(|(_, address)| {
                found
                    .get(address.as_str())
                    .is_none_or(|last_seen| *last_seen < cutoff)
            })
            .map(|(id, _)| id.to_string())
            .collect();

        offline.sort();

        Ok(offline)
    }

    #[lua(scope = Device::Read)]
    async fn metric(
        cx: &LuaCallContext,
        lua: &Lua,
        device: String,
        key: String,
    ) -> mlua::Result<Option<MetricSample>> {
        let address = cx.state.devices.address_or_self(&device).to_owned();

        let row = cx
            .query(Self::METRIC, || async {
                cx.state.repos.metric().latest(&address, &key).await
            })
            .await?;

        row.map(|row| MetricSample::new(lua, row)).transpose()
    }

    #[lua(scope = Device::Read)]
    async fn metric_since(
        cx: &LuaCallContext,
        lua: &Lua,
        device: String,
        key: String,
        epoch: i64,
    ) -> mlua::Result<Vec<MetricSample>> {
        let address = cx.state.devices.address_or_self(&device).to_owned();
        let earliest = DateTime::from_timestamp(epoch, 0)
            .ok_or_else(|| format!("epoch {epoch} is out of range").into_lua_err())?;

        let rows = cx
            .query(Self::METRIC_SINCE, || async {
                cx.state
                    .repos
                    .metric()
                    .since(&address, &key, earliest)
                    .await
            })
            .await?;

        rows.into_iter()
            .map(|row| MetricSample::new(lua, row))
            .collect()
    }
}
