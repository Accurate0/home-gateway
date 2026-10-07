use std::convert::Infallible;

use mlua::ExternalError;

use crate::integrations::notify::{Notification, notify};
use crate::lua::{Json, LuaCallContext, LuaClass, lua_module};
use crate::settings::{
    NotificationSource, NotifyAcknowledge, NotifyAction, NotifyActionKind, NotifyCategory,
    NotifySource, validate_acknowledge,
};

use super::actions;

#[derive(LuaClass)]
#[lua(name = "Notification", input)]
pub struct Request {
    message: String,
    title: Option<String>,
    category: Json<NotifyCategory>,
    tag: Option<String>,
    actions: Option<Vec<Json<NotifyAction>>>,
    acknowledge: Option<Json<NotifyAcknowledge>>,
}

pub struct NotifyLua;

#[lua_module(namespace = "notify")]
impl NotifyLua {
    #[lua(scope = Push::Write)]
    async fn send(cx: &LuaCallContext, notification: Request) -> mlua::Result<()> {
        let message = notification.message;
        let acknowledge = notification.acknowledge.map(|acknowledge| acknowledge.0);

        let declared: Vec<NotifyAction> = notification
            .actions
            .unwrap_or_default()
            .into_iter()
            .map(|action| action.0)
            .collect();

        let has_action = declared
            .iter()
            .any(|action| matches!(action.action, NotifyActionKind::Acknowledge));

        validate_acknowledge(acknowledge.is_some(), has_action).map_err(|e| e.into_lua_err())?;

        let resolved = actions::resolve(&cx.state.settings.workflows, &declared);

        if resolved.len() != declared.len() {
            return Err("a notify action names an unknown workflow".into_lua_err());
        }

        let tag = notification
            .tag
            .unwrap_or_else(|| format!("workflow:{}", cx.origin));

        let source = NotificationSource::Lua {
            origin: cx.origin.clone(),
        };

        let push = Notification::new(source, message.clone(), notification.category.0, tag)
            .with_actions(resolved)
            .with_acknowledge(acknowledge);

        let push = match notification.title {
            Some(title) => push.with_title(title),
            None => push,
        };

        cx.command(Self::SEND, &message, || async {
            notify(&[NotifySource::AndroidApp], push);

            Ok::<(), Infallible>(())
        })
        .await
    }
}
