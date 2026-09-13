use mlua::{Lua, Table};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType};

use super::MqttClient;

const PUBLISH: LuaFunction = LuaFunction {
    name: "publish",
    params: &[
        LuaParam {
            name: "topic",
            ty: LuaType::String,
        },
        LuaParam {
            name: "payload",
            ty: LuaType::String,
        },
        LuaParam {
            name: "retain",
            ty: LuaType::Optional(&LuaType::Boolean),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Mqtt, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[PUBLISH];

pub struct MqttLua;

impl LuaModule for MqttLua {
    fn namespace(&self) -> &'static str {
        "mqtt"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let publish_cx = cx.clone();

        cx.expose(table, &PUBLISH, || {
            lua.create_async_function(
                move |_, (topic, payload, retain): (String, String, Option<bool>)| {
                    let cx = publish_cx.clone();

                    async move {
                        let client = cx.state.handles.expect::<MqttClient>().clone();
                        let detail = format!("{topic} = {payload}");

                        cx.command("mqtt.publish", detail, || async {
                            client
                                .send_event_raw(topic, &payload, retain.unwrap_or(false))
                                .await
                        })
                        .await
                    }
                },
            )
        })
    }
}
