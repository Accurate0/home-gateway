use super::query::{environment_metric, query_environment};
use crate::lua::{Json, LuaCallContext, LuaClass, lua_module};
use crate::workflows::definition::EnvMetric;

#[derive(LuaClass)]
#[lua(output)]
pub struct EnvironmentReading {
    temperature: f64,
    humidity: Option<f64>,
    pressure: Option<f64>,
    lux: Option<f64>,
    uv_index: Option<f64>,
}

pub struct EnvironmentLua;

#[lua_module(namespace = "environment")]
impl EnvironmentLua {
    #[lua(scope = Environment::Read)]
    async fn get(
        cx: &LuaCallContext,
        device: String,
        metric: Json<EnvMetric>,
    ) -> mlua::Result<Option<f64>> {
        let sensor = cx.state.devices.address_or_self(&device).to_owned();

        let reading = cx
            .query(Self::GET, || async {
                query_environment(&sensor, cx.timeout()).await
            })
            .await?;

        Ok(reading.and_then(|reading| environment_metric(&reading, metric.0)))
    }

    #[lua(scope = Environment::Read)]
    async fn reading(
        cx: &LuaCallContext,
        device: String,
    ) -> mlua::Result<Option<EnvironmentReading>> {
        let sensor = cx.state.devices.address_or_self(&device).to_owned();

        let reading = cx
            .query(Self::READING, || async {
                query_environment(&sensor, cx.timeout()).await
            })
            .await?;

        Ok(reading.map(|reading| EnvironmentReading {
            temperature: reading.temperature,
            humidity: reading.humidity,
            pressure: reading.pressure,
            lux: reading.lux,
            uv_index: reading.uv_index,
        }))
    }
}
