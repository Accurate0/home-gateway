use mlua::{Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::integrations::notify::{Notification, notify};
use crate::lua::{
    LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType, schema,
};
use crate::settings::{NotifyCategory, NotifySource};

const NOTIFICATION: LuaClass = LuaClass {
    name: "Notification",
    fields: &[
        LuaField {
            name: "message",
            ty: LuaType::String,
        },
        LuaField {
            name: "title",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "category",
            ty: LuaType::Schema(schema::<NotifyCategory>),
        },
    ],
};

const SEND: LuaFunction = LuaFunction {
    name: "send",
    params: &[LuaParam {
        name: "notification",
        ty: LuaType::Class(&NOTIFICATION),
    }],
    returns: None,
    scope: Some(Scope::new(Resource::Push, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[SEND];

pub struct NotifyLua;

impl LuaModule for NotifyLua {
    fn namespace(&self) -> &'static str {
        "notify"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let send_cx = cx.clone();

        cx.expose(table, &SEND, || {
            lua.create_async_function(move |lua, request: Table| {
                let cx = send_cx.clone();

                async move {
                    let message: String = request.get("message")?;
                    let title: Option<String> = request.get("title")?;
                    let category: LuaValue = request.get("category")?;

                    let category: NotifyCategory = lua.from_value(category)?;
                    let tag = format!("workflow:{}", cx.origin);

                    let notification = Notification::new(message.clone(), category, tag);
                    let notification = match title {
                        Some(title) => notification.with_title(title),
                        None => notification,
                    };

                    cx.command("notify.send", &message, || async {
                        notify(&[NotifySource::AndroidApp], notification);

                        Ok::<(), std::convert::Infallible>(())
                    })
                    .await
                }
            })
        })
    }
}
