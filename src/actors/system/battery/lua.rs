use crate::lua::{LuaCallContext, LuaClass, lua_module};

#[derive(LuaClass)]
#[lua(output)]
pub struct LowBattery {
    device: String,
    level: f64,
}

pub struct BatteryLua;

#[lua_module(namespace = "battery")]
impl BatteryLua {
    #[lua(scope = Device::Read)]
    async fn level(cx: &LuaCallContext, device: String) -> mlua::Result<Option<f64>> {
        let keys = vec![cx.state.devices.address_or_self(&device).to_owned()];

        let rows = cx
            .query(Self::LEVEL, || async {
                cx.state.repos.battery().latest_many(&keys).await
            })
            .await?;

        Ok(rows.into_iter().find_map(|row| row.battery_percent))
    }

    #[lua(scope = Device::Read)]
    async fn low(cx: &LuaCallContext, threshold: f64) -> mlua::Result<Vec<LowBattery>> {
        let keys: Vec<String> = cx
            .state
            .devices
            .ids()
            .values()
            .map(|address| address.to_string())
            .collect();

        let rows = cx
            .query(Self::LOW, || async {
                cx.state.repos.battery().latest_many(&keys).await
            })
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let level = row.battery_percent.filter(|level| *level < threshold)?;

                let device = cx
                    .state
                    .devices
                    .id_for_address(&row.device_id)
                    .unwrap_or(&row.device_id)
                    .to_owned();

                Some(LowBattery { device, level })
            })
            .collect())
    }
}
