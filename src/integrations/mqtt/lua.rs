use crate::lua::{LuaCallContext, lua_module};

use super::MqttClient;

pub struct MqttLua;

#[lua_module(namespace = "mqtt")]
impl MqttLua {
    #[lua(scope = Mqtt::Write)]
    async fn publish(
        cx: &LuaCallContext,
        topic: String,
        payload: String,
        retain: Option<bool>,
    ) -> mlua::Result<()> {
        let client = cx.state.handles.expect::<MqttClient>().clone();
        let detail = format!("{topic} = {payload}");

        cx.command(Self::PUBLISH, detail, || async {
            client
                .send_event_raw(topic, &payload, retain.unwrap_or(false))
                .await
        })
        .await
    }
}
