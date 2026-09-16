use mlua::{Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::integrations::notify::{Notification, notify};
use crate::lua::{
    LuaCallContext, LuaClass, LuaField, LuaFunction, LuaModule, LuaParam, LuaType, schema,
};
use crate::settings::{
    NotifyAcknowledge, NotifyAction, NotifyActionKind, NotifyCategory, NotifySource,
    validate_acknowledge,
};

use super::actions;

const ACKNOWLEDGE: LuaClass = LuaClass {
    name: "NotifyAcknowledge",
    fields: &[
        LuaField {
            name: "remind_after",
            ty: LuaType::String,
        },
        LuaField {
            name: "reminders",
            ty: LuaType::Integer,
        },
    ],
};

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
        LuaField {
            name: "tag",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "actions",
            ty: LuaType::Optional(&LuaType::Array(&LuaType::Schema(schema::<NotifyAction>))),
        },
        LuaField {
            name: "acknowledge",
            ty: LuaType::Optional(&LuaType::Class(&ACKNOWLEDGE)),
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
                    let tag: Option<String> = request.get("tag")?;
                    let declared: Option<LuaValue> = request.get("actions")?;
                    let acknowledge: Option<LuaValue> = request.get("acknowledge")?;

                    let category: NotifyCategory = lua.from_value(category)?;

                    let declared: Vec<NotifyAction> = match declared {
                        Some(declared) => lua.from_value(declared)?,
                        None => Vec::new(),
                    };

                    let acknowledge: Option<NotifyAcknowledge> = match acknowledge {
                        Some(acknowledge) => Some(lua.from_value(acknowledge)?),
                        None => None,
                    };

                    let has_action = declared
                        .iter()
                        .any(|action| matches!(action.action, NotifyActionKind::Acknowledge));

                    validate_acknowledge(acknowledge.is_some(), has_action)
                        .map_err(mlua::Error::external)?;

                    let resolved = actions::resolve(&cx.state.settings.workflows, &declared);

                    if resolved.len() != declared.len() {
                        return Err(mlua::Error::external(
                            "a notify action names an unknown workflow",
                        ));
                    }

                    let tag = tag.unwrap_or_else(|| format!("workflow:{}", cx.origin));

                    let notification = Notification::new(message.clone(), category, tag)
                        .with_actions(resolved)
                        .with_acknowledge(acknowledge);

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
